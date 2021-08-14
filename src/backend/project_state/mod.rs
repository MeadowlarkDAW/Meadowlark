use basedrop::{Collector, Handle, Shared, SharedCell};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LockResult, Mutex,
};
use std::time::Duration;

use rusty_daw_time::{MusicalTime, SampleRate, Seconds, TempoMap};

use crate::backend::audio_graph::{CompiledGraph, GraphStateInterface, NodeID, PortType};
use crate::backend::generic_nodes;
use crate::backend::resource_loader::{ResourceLoadError, ResourceLoader};
use crate::backend::timeline::{
    AudioClipResourceCache, AudioClipSaveState, LoopState, TimelineTrackHandle,
    TimelineTrackSaveState, TimelineTransportHandle, TimelineTransportSaveState,
};

use super::timeline::TimelineTrackNode;

static COLLECT_INTERVAL: Duration = Duration::from_secs(3);

static DEFAULT_AUDIO_CLIP_DECLICK_TIME: Seconds = Seconds(2.0 / 1_000.0);

/// This struct should contain all information needed to create a "save file"
/// for the project.
///
/// TODO: Project file format. This will need to be future-proof.
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
            )],
        ));

        new_self
    }
}

/// All operations that affect the project state must happen through one of this struct's
/// methods. As such this struct just be responsible for checking that the project state
/// always remains valid. This will also allow us to create a scripting api later on.
pub struct ProjectStateInterface {
    save_state: ProjectSaveState,

    graph_interface: GraphStateInterface,

    resource_loader: Arc<Mutex<ResourceLoader>>,
    audio_clip_resource_cache: Arc<Mutex<AudioClipResourceCache>>,

    timeline_track_handles: Vec<TimelineTrackHandle>,
    timeline_track_node_ids: Vec<NodeID>,

    timeline_transport: TimelineTransportHandle,

    master_track_mix_in_node_id: NodeID,

    sample_rate: SampleRate,

    coll_handle: Handle,

    running: Arc<AtomicBool>,
}

impl ProjectStateInterface {
    pub fn new(
        mut save_state: ProjectSaveState,
        sample_rate: SampleRate,
    ) -> (
        Self,
        Shared<SharedCell<CompiledGraph>>,
        Vec<ResourceLoadError>,
    ) {
        save_state.tempo_map.sample_rate = sample_rate;

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
            GraphStateInterface::new(sample_rate, coll_handle.clone(), &&save_state);

        let mut master_track_mix_in_node_id = None;

        graph_interface.modify_graph(|mut graph| {
            for timeline_track_save in save_state.timeline_tracks.iter() {
                let (node, handle, mut res) = TimelineTrackNode::new(
                    timeline_track_save,
                    &resource_loader,
                    &audio_clip_resource_cache,
                    &save_state.tempo_map,
                    sample_rate,
                    coll_handle.clone(),
                );

                // Append any errors that happened while loading resources.
                load_errors.append(&mut res);

                let node_id = graph.add_new_node(Box::new(node));

                timeline_track_handles.push(handle);
                timeline_track_node_ids.push(node_id);
            }

            // All timeline tracks will be mixed into a single "master" track.
            //
            // TODO: Track routing.
            let master_track_mix_id = graph.add_new_node(Box::new(
                generic_nodes::mix::StereoMixNode::new(timeline_track_handles.len()),
            ));

            // Connect all timeline tracks to the "master" track.
            //
            // TODO: Track routing.
            for (i, node_id) in timeline_track_node_ids.iter().enumerate() {
                graph
                    .add_port_connection(PortType::StereoAudio, node_id, 0, &master_track_mix_id, i)
                    .unwrap();
            }

            master_track_mix_in_node_id = Some(master_track_mix_id);
        });

        (
            Self {
                save_state,

                graph_interface,

                resource_loader,
                audio_clip_resource_cache,

                timeline_track_handles,
                timeline_track_node_ids,

                timeline_transport,

                master_track_mix_in_node_id: master_track_mix_in_node_id.unwrap(),

                sample_rate,
                coll_handle,

                running,
            },
            rt_graph_interface,
            load_errors,
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
        let num_timeline_tracks = self.save_state.timeline_tracks.len();
        let master_track_mix_in_node_id = self.master_track_mix_in_node_id;

        self.graph_interface.modify_graph(|mut graph| {
            let n_id = graph.add_new_node(Box::new(node));

            // All timeline tracks will be mixed into a single "master" track.
            //
            // TODO: Track routing.
            //
            // Replace the current mix node with one that has the correct number of inputs.
            let master_mix_node = generic_nodes::mix::StereoMixNode::new(num_timeline_tracks);
            graph
                .replace_node(&master_track_mix_in_node_id, Box::new(master_mix_node))
                .unwrap();

            // Connect the new track to the "master" track;
            graph
                .add_port_connection(
                    PortType::StereoAudio,
                    &n_id,
                    0,
                    &master_track_mix_in_node_id,
                    num_timeline_tracks - 1,
                )
                .unwrap();

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

    pub fn timeline_transport(
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

    pub fn resource_loader(&self) -> &Arc<Mutex<ResourceLoader>> {
        &self.resource_loader
    }

    pub fn audio_clip_resource_cache(&self) -> &Arc<Mutex<AudioClipResourceCache>> {
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
