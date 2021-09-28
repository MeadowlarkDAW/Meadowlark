use audio_graph::NodeRef;
use basedrop::{Collector, Handle, Shared, SharedCell};
use rusty_daw_time::SampleRate;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LockResult, Mutex,
};
use std::time::Duration;
use tuix::Lens;

use crate::backend::graph::{CompiledGraph, GraphInterface};
use crate::backend::resource_loader::{ResourceLoadError, ResourceLoader};
use crate::backend::save_state::ProjectSaveState;
use crate::backend::timeline::{
    AudioClipResourceCache, TimelineTrackHandle, TimelineTrackNode, TimelineTrackSaveState,
    TimelineTransportHandle, TimelineTransportSaveState,
};

static COLLECT_INTERVAL: Duration = Duration::from_secs(3);

/// All operations that affect the project state must happen through one of this struct's
/// methods. As such this struct just be responsible for checking that the project state
/// always remains valid and up-to-date.
#[derive(Lens)]
pub struct BackendHandle {
    save_state: ProjectSaveState,

    graph_interface: GraphInterface,

    resource_loader: Arc<Mutex<ResourceLoader>>,
    audio_clip_resource_cache: Arc<Mutex<AudioClipResourceCache>>,

    timeline_track_handles: Vec<TimelineTrackHandle>,
    timeline_track_node_refs: Vec<NodeRef>,

    timeline_transport: TimelineTransportHandle,

    sample_rate: SampleRate,

    coll_handle: Handle,
    running: Arc<AtomicBool>,
}

impl BackendHandle {
    pub fn new(sample_rate: SampleRate) -> (Self, Shared<SharedCell<CompiledGraph>>) {
        let collector = Collector::new();
        let coll_handle = collector.handle();

        let resource_loader =
            Arc::new(Mutex::new(ResourceLoader::new(collector.handle(), sample_rate)));
        let resource_loader_clone = Arc::clone(&resource_loader);

        let audio_clip_resource_cache =
            Arc::new(Mutex::new(AudioClipResourceCache::new(collector.handle(), sample_rate)));
        let audio_clip_r_c_clone = Arc::clone(&audio_clip_resource_cache);

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        std::thread::spawn(|| {
            run_collector(collector, resource_loader_clone, audio_clip_r_c_clone, running_clone)
        });

        let (graph_interface, rt_graph_interface, timeline_transport) =
            GraphInterface::new(sample_rate, coll_handle.clone());

        (
            Self {
                save_state: ProjectSaveState::new_empty(sample_rate),

                graph_interface,

                resource_loader,
                audio_clip_resource_cache,

                timeline_track_handles: Vec::<TimelineTrackHandle>::new(),
                timeline_track_node_refs: Vec::<NodeRef>::new(),

                timeline_transport,

                sample_rate,
                coll_handle,

                running,
            },
            rt_graph_interface,
        )
    }

    pub fn project_save_state(&self) -> &ProjectSaveState {
        &self.save_state
    }

    // TODO: Interface for editing the tempo map directly.
    pub fn set_bpm(&mut self, bpm: f64) {
        assert!(bpm > 0.0);

        self.save_state.tempo_map.set_bpm(bpm);

        for (timeline_track, save_state) in
            self.timeline_track_handles.iter_mut().zip(self.save_state.timeline_tracks.iter())
        {
            timeline_track.update_tempo_map(&self.save_state.tempo_map, &save_state);
        }

        self.timeline_transport._update_tempo_map(self.save_state.tempo_map.clone());
    }

    /// Return an immutable handle to the timeline track with given index.
    pub fn timeline_track<'a>(
        &'a self,
        index: usize,
    ) -> Option<(&'a TimelineTrackHandle, &'a TimelineTrackSaveState)> {
        if let Some(timeline_track) = self.timeline_track_handles.get(index) {
            Some((timeline_track, &self.save_state.timeline_tracks[index]))
        } else {
            None
        }
    }

    /// Return a mutable handle to the timeline track with given index.
    pub fn timeline_track_mut<'a>(
        &'a mut self,
        index: usize,
    ) -> Option<(&'a mut TimelineTrackHandle, &'a mut TimelineTrackSaveState)> {
        if let Some(timeline_track) = self.timeline_track_handles.get_mut(index) {
            Some((timeline_track, &mut self.save_state.timeline_tracks[index]))
        } else {
            None
        }
    }

    pub fn add_timeline_track(
        &mut self,
        track: TimelineTrackSaveState,
    ) -> Result<(), Vec<ResourceLoadError>> {
        let mut load_errors = Vec::<ResourceLoadError>::new();

        let (node, handle, mut res) = TimelineTrackNode::new(
            &track,
            &self.resource_loader,
            &self.audio_clip_resource_cache,
            &self.save_state.tempo_map,
            self.sample_rate,
            self.coll_handle.clone(),
        );

        // Append any errors that happened while loading resources.
        load_errors.append(&mut res);

        self.timeline_track_handles.push(handle);
        self.save_state.timeline_tracks.push(track);

        let mut node_id = None;

        self.graph_interface.modify_graph(|mut graph| {
            let n_id = graph.add_new_node(Box::new(node));

            node_id = Some(n_id);
        });

        self.timeline_track_node_refs.push(node_id.unwrap());

        if load_errors.is_empty() {
            Ok(())
        } else {
            Err(load_errors)
        }
    }

    pub fn remove_timeline_track(&mut self, index: usize) -> Result<(), ()> {
        if index >= self.timeline_track_handles.len() {
            return Err(());
        }

        self.save_state.timeline_tracks.remove(index);
        self.timeline_track_handles.remove(index);

        let node_id = self.timeline_track_node_refs.remove(index);

        self.graph_interface.modify_graph(|mut graph| {
            graph.remove_node(node_id).unwrap();
        });

        Ok(())
    }

    pub fn get_timeline_transport(
        &mut self,
    ) -> (&mut TimelineTransportHandle, &mut TimelineTransportSaveState) {
        (&mut self.timeline_transport, &mut self.save_state.timeline_transport)
    }

    pub fn get_resource_loader(&self) -> &Arc<Mutex<ResourceLoader>> {
        &self.resource_loader
    }

    pub fn get_audio_clip_resource_cache(&self) -> &Arc<Mutex<AudioClipResourceCache>> {
        &self.audio_clip_resource_cache
    }
}

impl Drop for BackendHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn run_collector(
    mut collector: Collector,
    resource_loader: Arc<Mutex<ResourceLoader>>,
    audio_clip_resource_cache: Arc<Mutex<AudioClipResourceCache>>,
    running: Arc<AtomicBool>,
) {
    while running.load(Ordering::Relaxed) {
        std::thread::sleep(COLLECT_INTERVAL);

        {
            match audio_clip_resource_cache.lock() {
                LockResult::Ok(mut cache) => {
                    cache.collect();
                }
                LockResult::Err(e) => {
                    log::error!("{}", e);
                    break;
                }
            }
        }

        {
            match resource_loader.lock() {
                LockResult::Ok(mut res_loader) => {
                    res_loader.collect();
                }
                LockResult::Err(e) => {
                    log::error!("{}", e);
                    break;
                }
            }
        }

        collector.collect();
    }
    log::info!("shutting down collector");
}
