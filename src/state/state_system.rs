use std::collections::VecDeque;

use cpal::Stream;
use rusty_daw_audio_graph::{GraphStateRef, NodeRef, PortType};
use rusty_daw_core::{MusicalTime, SampleRate, SuperFrames};
use vizia::{Lens, Model};

use crate::backend::timeline::{TimelineTrackHandle, TimelineTrackNode, TimelineTrackState};
use crate::backend::{
    BackendCoreHandle, GlobalNodeData, ResourceCache, ResourceLoadError, MAX_BLOCKSIZE,
};

use super::ui_state::{LoopUiState, TimelineTrackUiState, UiState};
use super::ProjectSaveState;

pub struct Project {
    stream: Stream,
    pub save_state: ProjectSaveState,
    backend_core_handle: BackendCoreHandle,
    pub timeline_track_handles: Vec<(NodeRef, TimelineTrackHandle<MAX_BLOCKSIZE>)>,
}

/// This struct is responsible for managing and mutating state of the entire application.
/// All mutation of any backend, UI, or save state must happen through this struct.
/// It is the responsibility of this struct to make sure all 3 of these states are synced
/// properly.
#[derive(Lens)]
pub struct StateSystem {
    #[lens(ignore)]
    project: Option<Project>,
    ui_state: UiState,
    undo_stack: VecDeque<AppEvent>,

    #[lens(ignore)]
    next_empty_track_num: usize,
}

impl StateSystem {
    pub fn new() -> Self {
        Self {
            project: None,
            ui_state: UiState::default(),
            undo_stack: VecDeque::new(),
            next_empty_track_num: 1,
        }
    }

    pub fn get_project(&self) -> Option<&Project> {
        if let Some(project) = &self.project {
            Some(project)
        } else {
            None
        }
    }

    pub fn get_ui_state(&self) -> &UiState {
        &self.ui_state
    }

    pub fn load_project(&mut self, project_state: &Box<ProjectSaveState>) {
        // This will drop and automatically close any already active project.
        self.project = None;

        // Reset the UI state:
        self.ui_state.backend_loaded = false;
        self.ui_state.timeline_transport.is_playing = false;
        self.ui_state.timeline_tracks.clear();

        // This function is temporary. Eventually we should use rusty-daw-io instead.
        let sample_rate =
            crate::backend::hardware_io::default_sample_rate().unwrap_or(SampleRate::default());

        let mut save_state = ProjectSaveState {
            backend_core: project_state.backend_core.clone_with_sample_rate(sample_rate),
            timeline_tracks: Vec::with_capacity(project_state.timeline_tracks.len()),
        };

        let (mut backend_core_handle, rt_state) =
            BackendCoreHandle::from_state(sample_rate, &mut save_state.backend_core);

        let mut timeline_track_handles: Vec<(NodeRef, TimelineTrackHandle<MAX_BLOCKSIZE>)> =
            Vec::new();
        let mut resource_load_errors: Vec<ResourceLoadError> = Vec::new();

        //This function is temporary. Eventually we should use rusty-daw-io instead.
        if let Ok(stream) = crate::backend::rt_thread::run_with_default_output(rt_state) {
            self.ui_state.tempo_map.bpm = project_state.backend_core.tempo_map.bpm();
            self.ui_state.sample_rate = sample_rate;
            self.ui_state.timeline_transport.seek_to =
                project_state.backend_core.timeline_transport.seek_to;
            self.ui_state.timeline_transport.loop_state =
                project_state.backend_core.timeline_transport.loop_state.into();

            // TODO: errors and reverting to previous working state
            backend_core_handle
                .modify_graph(|mut graph, resource_cache| {
                    for timeline_track_state in project_state.timeline_tracks.iter() {
                        // --- Load timeline track in backend ----------------------
                        add_track_to_graph(
                            timeline_track_state.clone(),
                            &mut save_state,
                            &mut timeline_track_handles,
                            &mut graph,
                            resource_cache,
                            &mut resource_load_errors,
                        );

                        // --- Load timeline track in UI ---------------------------

                        self.ui_state.timeline_tracks.push(TimelineTrackUiState {
                            name: timeline_track_state.name.clone(),
                            height: 150.0,
                            audio_clips: timeline_track_state
                                .audio_clips
                                .iter()
                                .map(|s| s.into())
                                .collect(),
                        });
                    }
                })
                .unwrap();

            for e in resource_load_errors.iter() {
                // TODO: Alert user of resources that failed to load from disk
            }

            self.project =
                Some(Project { stream, save_state, backend_core_handle, timeline_track_handles });
            self.ui_state.backend_loaded = true;
        } else {
            // TODO: Better errors
            log::error!("Failed to start audio stream");
            // TODO: Remove this panic
            panic!("Failed to start audio stream");
        }
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        let bpm = if bpm <= 0.0 { 0.1 } else { bpm.clamp(0.0, 100_000.0) };
        self.ui_state.tempo_map.bpm = bpm;

        if let Some(project) = &mut self.project {
            project.backend_core_handle.set_bpm(bpm, &mut project.save_state.backend_core);
        }
    }

    pub fn set_loop_state(&mut self, mut loop_state: LoopUiState) {
        // Make the minimum loop length 1/16 of a beat.
        if loop_state.loop_end < loop_state.loop_start + MusicalTime::from_sixteenth_beats(0, 1) {
            loop_state.loop_end = loop_state.loop_start + MusicalTime::from_sixteenth_beats(0, 1);
        }
        self.ui_state.timeline_transport.loop_state = loop_state;
        self.ui_state.timeline_transport.seek_to = loop_state.loop_start;

        if let Some(project) = &mut self.project {
            let (transport, _) = project
                .backend_core_handle
                .timeline_transport_mut(&mut project.save_state.backend_core);

            if let Err(_) = transport.set_loop_state(
                loop_state.to_backend_state(),
                &mut project.save_state.backend_core.timeline_transport,
            ) {
                // TODO: Handle this.
            }
        }
    }

    // Set the start position of a clip in musical time
    pub fn set_clip_start(&mut self, track_id: usize, clip_id: usize, timeline_start: MusicalTime) {
        if let Some(track_state) = self.ui_state.timeline_tracks.get_mut(track_id) {
            if let Some(clip_state) = track_state.audio_clips.get_mut(clip_id) {
                clip_state.timeline_start = timeline_start;
            }
        };

        if let Some(project) = &mut self.project {
            if let Some((_, track)) = project.timeline_track_handles.get_mut(track_id) {
                let (tempo_map, timeline_tracks_state) = project.save_state.timeline_tracks_mut();
                if let Some((clip_handle, clip_state)) =
                    track.audio_clip_mut(clip_id, timeline_tracks_state.get_mut(track_id).unwrap())
                {
                    clip_handle.set_timeline_start(timeline_start, tempo_map, clip_state);
                }
            }
        }
    }

    /*
    pub fn trim_clip_start(
        &mut self,
        track_id: usize,
        clip_id: usize,
        timeline_start: MusicalTime,
    ) {
        if let Some(track_state) = self.ui_state.timeline_tracks.get_mut(track_id) {
            if let Some(clip_state) = track_state.audio_clips.get_mut(clip_id) {
                clip_state.timeline_start = timeline_start;
            }
        };

        if let Some(project) = &mut self.project {
            if let Some((_, track)) = project.timeline_track_handles.get_mut(track_id) {
                let (tempo_map, timeline_tracks_state) = project.save_state.timeline_tracks_mut();
                if let Some((clip_handle, clip_state)) =
                    track.audio_clip_mut(clip_id, timeline_tracks_state.get_mut(track_id).unwrap())
                {
                    let offset =
                        (timeline_start - clip_state.timeline_start).to_seconds(tempo_map.bpm());
                    clip_handle.set_clip_start_offset(
                        clip_state.clip_start_offset + offset,
                        tempo_map,
                        clip_state,
                    );
                    clip_handle.set_timeline_start(timeline_start, tempo_map, clip_state);
                }
            }
        }
    }
    */

    pub fn set_clip_end(&mut self, track_id: usize, clip_id: usize, clip_end: MusicalTime) {
        if let Some(track_state) = self.ui_state.timeline_tracks.get_mut(track_id) {
            if let Some(clip_state) = track_state.audio_clips.get_mut(clip_id) {
                clip_state.duration = if clip_state.timeline_start > clip_end {
                    SuperFrames(0)
                } else {
                    (clip_end.checked_sub(clip_state.timeline_start).unwrap())
                        .to_nearest_super_frame_round(self.ui_state.tempo_map.bpm)
                };

                if let Some(project) = &mut self.project {
                    if let Some((_, track)) = project.timeline_track_handles.get_mut(track_id) {
                        let (tempo_map, timeline_tracks_state) =
                            project.save_state.timeline_tracks_mut();
                        if let Some((clip_handle, clip_state)) = track.audio_clip_mut(
                            clip_id,
                            timeline_tracks_state.get_mut(track_id).unwrap(),
                        ) {
                            clip_handle.set_duration(clip_state.duration, tempo_map, clip_state);
                        }
                    }
                }
            }
        }
    }

    pub fn timeline_transport_seek_to(&mut self, seek: MusicalTime) {
        if let Some(project) = &mut self.project {
            self.ui_state.timeline_transport.seek_to = seek;

            let (transport, transport_state) = project
                .backend_core_handle
                .timeline_transport_mut(&mut project.save_state.backend_core);
            transport.seek_to(self.ui_state.timeline_transport.seek_to, transport_state);
        }
    }

    pub fn timeline_transport_play(&mut self) {
        if let Some(project) = &mut self.project {
            if !self.ui_state.timeline_transport.is_playing {
                self.ui_state.timeline_transport.is_playing = true;

                let (transport, transport_state) = project
                    .backend_core_handle
                    .timeline_transport_mut(&mut project.save_state.backend_core);
                transport.seek_to(self.ui_state.timeline_transport.seek_to, transport_state);
                transport.set_playing(true);
            }
        }
    }

    pub fn timeline_transport_pause(&mut self) {
        if let Some(project) = &mut self.project {
            if self.ui_state.timeline_transport.is_playing {
                self.ui_state.timeline_transport.is_playing = false;

                let (transport, _) = project
                    .backend_core_handle
                    .timeline_transport_mut(&mut project.save_state.backend_core);
                transport.set_playing(false);
            }
        }
    }

    /// Switch the timeline transport state between playing and paused
    pub fn timeline_transport_play_pause(&mut self) {
        if let Some(project) = &mut self.project {
            if self.ui_state.timeline_transport.is_playing {
                self.ui_state.timeline_transport.is_playing = false;

                let (transport, _) = project
                    .backend_core_handle
                    .timeline_transport_mut(&mut project.save_state.backend_core);
                transport.set_playing(false);
            } else {
                self.ui_state.timeline_transport.is_playing = true;
                let (transport, transport_state) = project
                    .backend_core_handle
                    .timeline_transport_mut(&mut project.save_state.backend_core);
                transport.seek_to(self.ui_state.timeline_transport.seek_to, transport_state);
                transport.set_playing(true);
            }
        }
    }

    pub fn timeline_transport_stop(&mut self) {
        self.ui_state.timeline_transport.is_playing = false;
        self.ui_state.timeline_transport.seek_to = MusicalTime::new(0, 0);

        if let Some(project) = &mut self.project {
            let (transport, transport_state) = project
                .backend_core_handle
                .timeline_transport_mut(&mut project.save_state.backend_core);
            transport.set_playing(false);
            transport.seek_to(MusicalTime::new(0, 0), transport_state);
        }
    }

    /*
    // Duplicates the timeline selection and places it after the selected region
    pub fn timeline_duplicate_selection(
        &mut self,
        track_id: usize,
        select_start: MusicalTime,
        select_end: MusicalTime,
    ) {
        // Collect all the clips (and parts of clips) that fall into the selection
        // Create new clips with those selected regions
        // For now I'm just going to copy whole clips but we can get this working later

        // TODO - get bpm
        let bpm = self.ui_state.tempo_map.bpm;
        let mut selected_clips = if let Some(track) = self.ui_state.timeline_tracks.get(track_id) {
            track
                .audio_clips
                .iter()
                .filter(|clip| {
                    clip.timeline_start >= select_start
                        && (clip.timeline_start + clip.duration.to_musical(bpm)) <= select_end
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        for selected_clip in selected_clips.iter_mut() {
            selected_clip.timeline_start =
                (selected_clip.timeline_start - select_start) + select_end;

            if let Some(project) = &mut self.project {
                if let Some((_, track)) = project.timeline_track_handles.get_mut(track_id) {
                    let (tempo_map, timeline_tracks_state) =
                        project.save_state.timeline_tracks_mut();

                    let clip_state = AudioClipState {
                        name: selected_clip.name.clone(),
                        pcm_path: selected_clip.pcm_path.clone(),
                        timeline_start: selected_clip.timeline_start,
                        duration: selected_clip.duration,
                        clip_start_offset: selected_clip.clip_start_offset,
                        clip_gain_db: selected_clip.clip_gain_db,
                        fades: AudioClipFades {
                            start_fade_duration: selected_clip.fades.start_fade_duration,
                            end_fade_duration: selected_clip.fades.end_fade_duration,
                        },
                    };

                    if let Err(e) = track.add_audio_clip(
                        clip_state,
                        project.backend_core_handle.resource_cache(),
                        tempo_map,
                        timeline_tracks_state.get_mut(track_id).unwrap(),
                    ) {
                        // TODO: Alert user that there was an error loading the resource from disk.
                    }
                }
            }
        }

        if let Some(track) = self.ui_state.timeline_tracks.get_mut(track_id) {
            track.audio_clips.extend(selected_clips.into_iter());
        }
    }
    */

    /*
    // Removes a timeline selection
    pub fn timeline_remove_selection(
        &mut self,
        track_id: usize,
        select_start: MusicalTime,
        select_end: MusicalTime,
    ) {
        // TODO - get bpm
        let bpm = self.ui_state.tempo_map.bpm;
        let mut selected_clips = if let Some(track) = self.ui_state.timeline_tracks.get(track_id) {
            track
                .audio_clips
                .iter()
                .cloned()
                .enumerate()
                .filter(|(clip_id, clip)| {
                    clip.timeline_start >= select_start
                        && (clip.timeline_start + clip.duration.to_musical(bpm)) <= select_end
                })
                .collect::<Vec<(usize, AudioClipUiState)>>()
        } else {
            Vec::new()
        };

        for (clip_id, selected_clip) in selected_clips.iter_mut() {
            selected_clip.timeline_start =
                (selected_clip.timeline_start - select_start) + select_end;

            if let Some(project) = &mut self.project {
                if let Some((_, track)) = project.timeline_track_handles.get_mut(track_id) {
                    let (_, timeline_tracks_state) = project.save_state.timeline_tracks_mut();

                    if let Err(_) = track.remove_audio_clip(
                        *clip_id,
                        timeline_tracks_state.get_mut(track_id).unwrap(),
                    ) {
                        // TODO: Handle error
                    }
                }
            }

            if let Some(track) = self.ui_state.timeline_tracks.get_mut(track_id) {
                track.audio_clips.remove(*clip_id);
            }
        }
    }
    */

    pub fn add_track(&mut self, new_track_state: TimelineTrackState) {
        // --- Load timeline track in UI ---------------------------
        self.ui_state.timeline_tracks.push(TimelineTrackUiState {
            name: new_track_state.name.clone(),
            height: 150.0,
            audio_clips: new_track_state.audio_clips.iter().map(|s| s.into()).collect(),
        });

        // --- Load timeline track in backend ----------------------
        if let Some(project) = &mut self.project {
            let mut resource_load_errors: Vec<ResourceLoadError> = Vec::new();

            let Project { save_state, backend_core_handle, timeline_track_handles, .. } = project;

            // TODO: errors and reverting to previous working state
            backend_core_handle
                .modify_graph(|mut graph, resource_cache| {
                    add_track_to_graph(
                        new_track_state,
                        save_state,
                        timeline_track_handles,
                        &mut graph,
                        resource_cache,
                        &mut resource_load_errors,
                    );
                })
                .unwrap();

            for e in resource_load_errors.iter() {
                // TODO: Alert user of resources that failed to load from disk
            }
        }
    }

    pub fn sync_playhead(&mut self) {
        if let Some(project) = &mut self.project {
            let (transport, _) = project
                .backend_core_handle
                .timeline_transport_mut(&mut project.save_state.backend_core);
            self.ui_state.timeline_transport.playhead = transport.get_playhead_position();
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
    // Force a sync
    Sync,

    // Tempo Controls
    SetBpm(f64),

    // Timeline Contols
    // TODO - Add track_start and track_end to this
    // TODO - Maybe change the name of this
    DuplicateSelection(usize, MusicalTime, MusicalTime),
    RemoveSelection(usize, MusicalTime, MusicalTime),

    // Transport Controls
    Play,
    Pause,
    PlayPause,
    Stop,
    SeekTo(MusicalTime),

    // Loop Controls
    SetLoopState(LoopUiState),

    // Track Controls
    AddTrack, // TODO - add data needed for adding track before/after another track
    SetTrackHeight(usize, f32),

    // Clip Controls
    // TODO - create types for track id and clip id
    SetClipStart(usize, usize, MusicalTime),
    SetClipEnd(usize, usize, MusicalTime),
    TrimClipStart(usize, usize, MusicalTime),
    TrimClipEnd(usize, usize, MusicalTime),
}

impl Model for StateSystem {
    fn event(&mut self, cx: &mut vizia::Context, event: &mut vizia::Event) {
        if let Some(app_event) = event.message.downcast() {
            match app_event {
                AppEvent::Sync => {
                    self.sync_playhead();
                }

                // TEMPO
                AppEvent::SetBpm(bpm) => {
                    self.set_bpm(*bpm);
                }

                // TIMELINE
                AppEvent::DuplicateSelection(track_id, select_start, select_end) => {
                    //self.timeline_duplicate_selection(*track_id, *select_start, *select_end);
                }

                AppEvent::RemoveSelection(track_id, select_start, select_end) => {
                    //self.timeline_remove_selection(*track_id, *select_start, *select_end);
                }

                // TRANSPORT
                AppEvent::Play => {
                    self.timeline_transport_play();
                    self.sync_playhead();
                }

                AppEvent::Pause => {
                    self.timeline_transport_pause();
                    self.sync_playhead();
                }

                AppEvent::PlayPause => {
                    self.timeline_transport_play_pause();
                    self.sync_playhead();
                }

                AppEvent::Stop => {
                    self.timeline_transport_stop();
                    self.sync_playhead();
                }

                AppEvent::SeekTo(time) => {
                    self.timeline_transport_seek_to(*time);
                }

                // LOOP
                AppEvent::SetLoopState(loop_state) => {
                    self.set_loop_state(*loop_state);
                }

                // TRACK
                AppEvent::AddTrack => {
                    let name = format!("Track {}", self.next_empty_track_num);
                    self.next_empty_track_num += 1;

                    self.add_track(TimelineTrackState { name, audio_clips: Vec::new() });
                }

                AppEvent::SetTrackHeight(track_id, track_height) => {
                    if let Some(track_state) = self.ui_state.timeline_tracks.get_mut(*track_id) {
                        track_state.height = *track_height;
                    }
                }

                // CLIP
                AppEvent::SetClipStart(track_id, clip_id, timeline_start) => {
                    self.set_clip_start(*track_id, *clip_id, *timeline_start);
                }

                AppEvent::SetClipEnd(track_id, clip_id, timeline_start) => {
                    self.set_clip_end(*track_id, *clip_id, *timeline_start);
                }

                AppEvent::TrimClipStart(track_id, clip_id, timeline_start) => {
                    //let timeline_start = MusicalTime::new(timeline_start.0.max(0.0));
                    //self.trim_clip_start(*track_id, *clip_id, timeline_start);
                }

                AppEvent::TrimClipEnd(track_id, clip_id, timeline_start) => {
                    self.set_clip_end(*track_id, *clip_id, *timeline_start);
                }
            }
        }
    }
}

fn add_track_to_graph(
    new_track_state: TimelineTrackState,
    project_save_state: &mut ProjectSaveState,
    timeline_track_handles: &mut Vec<(NodeRef, TimelineTrackHandle<MAX_BLOCKSIZE>)>,
    graph: &mut GraphStateRef<GlobalNodeData, MAX_BLOCKSIZE>,
    resource_cache: &ResourceCache,
    resource_load_errors: &mut Vec<ResourceLoadError>,
) {
    let root_node_ref = graph.root_node();

    let (tempo_map, tracks) = project_save_state.timeline_tracks_mut();

    let (timeline_track_node, timeline_track_handle, mut res) =
        TimelineTrackNode::new(&new_track_state, resource_cache, &tempo_map, graph.coll_handle());

    tracks.push(new_track_state);

    // Append any errors while loading resources from disk.
    resource_load_errors.append(&mut res);

    // Add the track node to the graph.
    let timeline_track_node_ref = graph.add_new_node(Box::new(timeline_track_node));

    // Keep a reference and a handle to the track node.
    timeline_track_handles.push((timeline_track_node_ref, timeline_track_handle));

    // Connect the track node to the root node.
    //
    // TODO: Handle errors.
    graph
        .connect_ports(PortType::StereoAudio, timeline_track_node_ref, 0, root_node_ref, 0)
        .unwrap();
}
