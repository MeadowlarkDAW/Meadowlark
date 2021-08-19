use basedrop::{Collector, Handle, Shared, SharedCell};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LockResult, Mutex,
};
use std::time::Duration;

use rusty_daw_time::{MusicalTime, SampleRate, Seconds, TempoMap};

use crate::backend::generic_nodes;
use crate::backend::graph::{CompiledGraph, GraphStateInterface, NodeID, PortType};
use crate::backend::resource_loader::{ResourceLoadError, ResourceLoader};
use crate::backend::timeline::{
    audio_clip::DEFAULT_AUDIO_CLIP_DECLICK_TIME, AudioClipResourceCache, AudioClipSaveState,
    LoopState, TimelineTrackHandle, TimelineTrackSaveState, TimelineTransportHandle,
    TimelineTransportSaveState,
};

use super::timeline::TimelineTrackNode;

use tuix::Lens;

static COLLECT_INTERVAL: Duration = Duration::from_secs(3);

/// This struct should contain all information needed to create a "save file"
/// for the project.
///
/// TODO: Project file format. This will need to be future-proof.
#[derive(Lens)]
pub struct ProjectSaveState {
    pub timeline_tracks: Vec<TimelineTrackSaveState>,
    pub timeline_transport: TimelineTransportSaveState,
    pub tempo_map: TempoMap,
    pub audio_clip_declick_time: Seconds,
}

impl ProjectSaveState {
    pub fn new_empty(sample_rate: SampleRate) -> Self {
        Self {
            timeline_tracks: Vec::new(),
            timeline_transport: Default::default(),
            tempo_map: TempoMap::new(110.0, sample_rate.into()),
            audio_clip_declick_time: DEFAULT_AUDIO_CLIP_DECLICK_TIME,
        }
    }

    pub fn test(sample_rate: SampleRate) -> Self {
        let mut new_self = ProjectSaveState::new_empty(sample_rate);

        new_self.timeline_transport.loop_state = LoopState::Active {
            loop_start: MusicalTime::new(0.0),
            loop_end: MusicalTime::new(4.0),
        };

        new_self.timeline_tracks.push(TimelineTrackSaveState::new(
            String::from("Track 1"),
            vec![AudioClipSaveState::new(
                String::from("Audio Clip 1"),
                "./test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                MusicalTime::new(0.0),
                Seconds::new(3.0),
                Seconds::new(0.0),
                -3.0,
                Default::default(),
            )],
        ));

        new_self.timeline_tracks.push(TimelineTrackSaveState::new(
            String::from("Track 2"),
            vec![AudioClipSaveState::new(
                String::from("Audio Clip 1"),
                "./test_files/synth_keys/synth_keys_48000_16bit.wav".into(),
                MusicalTime::new(0.0),
                Seconds::new(3.0),
                Seconds::new(0.0),
                -3.0,
                Default::default(),
            )],
        ));

        new_self
    }
}

/// All operations that affect the project state must happen through one of this struct's
/// methods. As such this struct just be responsible for checking that the project state
/// always remains valid. This will also allow us to create a scripting api later on.

#[derive(Lens)]
pub struct ProjectStateInterface {
    save_state: ProjectSaveState,

    graph_interface: GraphStateInterface,

    resource_loader: Arc<Mutex<ResourceLoader>>,
    audio_clip_resource_cache: Arc<Mutex<AudioClipResourceCache>>,

    timeline_track_handles: Vec<TimelineTrackHandle>,
    timeline_track_node_ids: Vec<NodeID>,

    timeline_transport: TimelineTransportHandle,

    sample_rate: SampleRate,

    coll_handle: Handle,

    running: Arc<AtomicBool>,
}

impl ProjectStateInterface {
    pub fn new(sample_rate: SampleRate) -> (Self, Shared<SharedCell<CompiledGraph>>) {
        let collector = Collector::new();
        let coll_handle = collector.handle();

        let resource_loader = Arc::new(Mutex::new(ResourceLoader::new(
            collector.handle(),
            sample_rate,
        )));
        let resource_loader_clone = Arc::clone(&resource_loader);

        let audio_clip_resource_cache = Arc::new(Mutex::new(AudioClipResourceCache::new(
            collector.handle(),
            sample_rate,
        )));
        let audio_clip_r_c_clone = Arc::clone(&audio_clip_resource_cache);

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        std::thread::spawn(|| {
            run_collector(
                collector,
                resource_loader_clone,
                audio_clip_r_c_clone,
                running_clone,
            )
        });

        let mut load_errors = Vec::<ResourceLoadError>::new();
        let mut timeline_track_handles = Vec::<TimelineTrackHandle>::new();
        let mut timeline_track_node_ids = Vec::<NodeID>::new();

        let (mut graph_interface, rt_graph_interface, timeline_transport) =
            GraphStateInterface::new(sample_rate, coll_handle.clone());

        (
            Self {
                save_state: ProjectSaveState::new_empty(sample_rate),

                graph_interface,

                resource_loader,
                audio_clip_resource_cache,

                timeline_track_handles,
                timeline_track_node_ids,

                timeline_transport,

                sample_rate,
                coll_handle,

                running,
            },
            rt_graph_interface,
        )
    }

    // TODO: Interface for editing the tempo map directly.
    pub fn set_bpm(&mut self, bpm: f64) {
        assert!(bpm > 0.0);

        self.save_state.tempo_map.set_bpm(bpm);

        for (timeline_track, save_state) in self
            .timeline_track_handles
            .iter_mut()
            .zip(self.save_state.timeline_tracks.iter())
        {
            timeline_track.update_tempo_map(&self.save_state.tempo_map, &save_state);
        }

        self.timeline_transport
            ._update_tempo_map(self.save_state.tempo_map.clone());
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

        self.timeline_track_node_ids.push(node_id.unwrap());

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

        let node_id = self.timeline_track_node_ids.remove(index);

        self.graph_interface.modify_graph(|mut graph| {
            graph.remove_node(&node_id).unwrap();
        });

        Ok(())
    }

    pub fn get_timeline_transport(
        &mut self,
    ) -> (
        &mut TimelineTransportHandle,
        &mut TimelineTransportSaveState,
    ) {
        (
            &mut self.timeline_transport,
            &mut self.save_state.timeline_transport,
        )
    }

    pub fn get_resource_loader(&self) -> &Arc<Mutex<ResourceLoader>> {
        &self.resource_loader
    }

    pub fn get_audio_clip_resource_cache(&self) -> &Arc<Mutex<AudioClipResourceCache>> {
        &self.audio_clip_resource_cache
    }
}

impl Drop for ProjectStateInterface {
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
