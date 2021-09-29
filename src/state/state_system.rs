use cpal::Stream;
use rusty_daw_time::SampleRate;
//use std::collections::VecDeque;
use tuix::PropSet;
use tuix::{BindEvent, Entity, State};

use crate::backend::resource_loader::ResourceLoadError;
use crate::backend::{BackendHandle, BackendSaveState};

use super::event::*;
use super::{BoundGuiState, ProjectSaveState};

//const EVENT_QUEUE_INITIAL_SIZE: usize = 256;

pub struct StateSystem {
    stream: Option<Stream>,
    backend_handle: Option<BackendHandle>,
    backend_save_state: BackendSaveState,
    //event_queue: VecDeque<StateSystemEvent>,
}

impl StateSystem {
    pub fn new() -> Self {
        Self {
            stream: None,
            backend_handle: None,
            backend_save_state: BackendSaveState::new(SampleRate(48_000.0)),
            //event_queue: VecDeque::with_capacity(EVENT_QUEUE_INITIAL_SIZE),
        }
    }

    pub fn on_event(
        &mut self,
        bound_gui_state: &mut BoundGuiState,
        state: &mut State,
        entity: Entity,
        event: &mut StateSystemEvent,
    ) {
        match event {
            StateSystemEvent::Transport(event) => {
                self.on_transport_event(bound_gui_state, state, entity, event)
            }
            StateSystemEvent::Tempo(event) => {
                self.on_tempo_event(bound_gui_state, state, entity, event)
            }
            StateSystemEvent::Project(event) => {
                self.on_project_event(bound_gui_state, state, entity, event)
            }
        }
    }

    pub fn on_tempo_event(
        &mut self,
        bound_gui_state: &mut BoundGuiState,
        state: &mut State,
        entity: Entity,
        event: &mut TempoEvent,
    ) {
        if let Some(backend_handle) = &mut self.backend_handle {
            match event {
                TempoEvent::SetBPM(bpm) => {
                    let bpm = if *bpm <= 0.0 { 0.1 } else { bpm.clamp(0.0, 100_000.0) };

                    bound_gui_state.bpm = bpm;
                    entity.emit(state, BindEvent::Update);

                    backend_handle.set_bpm(bpm, &mut self.backend_save_state)
                }
            }
        }
    }

    pub fn on_transport_event(
        &mut self,
        bound_gui_state: &mut BoundGuiState,
        state: &mut State,
        entity: Entity,
        event: &mut TransportEvent,
    ) {
        if let Some(backend_handle) = &mut self.backend_handle {
            match event {
                TransportEvent::Play => {
                    if !bound_gui_state.is_playing {
                        bound_gui_state.is_playing = true;
                        entity.emit(state, BindEvent::Update);

                        let (transport, _) =
                            backend_handle.get_timeline_transport(&mut self.backend_save_state);
                        transport.set_playing(true);
                    }
                }
                TransportEvent::Stop => {
                    if bound_gui_state.is_playing {
                        bound_gui_state.is_playing = false;
                        entity.emit(state, BindEvent::Update);
                    }

                    let (transport, save_state) =
                        backend_handle.get_timeline_transport(&mut self.backend_save_state);
                    transport.set_playing(false);
                    // TODO: have the transport struct handle this.
                    transport.seek_to(0.0.into(), save_state);
                }
                TransportEvent::Pause => {
                    if bound_gui_state.is_playing {
                        bound_gui_state.is_playing = false;
                        entity.emit(state, BindEvent::Update);

                        let (transport, _) =
                            backend_handle.get_timeline_transport(&mut self.backend_save_state);
                        transport.set_playing(false);
                    }
                }
            }
        }
    }

    pub fn on_project_event(
        &mut self,
        bound_gui_state: &mut BoundGuiState,
        state: &mut State,
        entity: Entity,
        event: &mut ProjectEvent,
    ) {
        match event {
            ProjectEvent::LoadProject(project_save_state) => {
                self.load_project(bound_gui_state, project_save_state, state, entity)
            }
        }
    }

    fn load_project(
        &mut self,
        bound_gui_state: &mut BoundGuiState,
        project_save_state: &Box<ProjectSaveState>,
        state: &mut State,
        entity: Entity,
    ) {
        let mut update_gui = || {
            entity.emit(state, BindEvent::Update);
        };

        // Reset all events
        //self.event_queue.clear();

        bound_gui_state.backend_loaded = false;
        bound_gui_state.is_playing = false;
        update_gui();

        // This will drop and automatically close any active backend/stream.
        self.backend_handle = None;
        self.stream = None;

        // This function is temporary. Eventually we should use rusty-daw-io instead.
        let sample_rate =
            crate::backend::hardware_io::default_sample_rate().unwrap_or(SampleRate(48_000.0));

        let (mut backend_handle, rt_state) = BackendHandle::new(sample_rate);

        self.backend_save_state = BackendSaveState::new(sample_rate);

        let mut resource_load_errors: Vec<ResourceLoadError> = Vec::new();

        // This function is temporary. Eventually we should use rusty-daw-io instead.
        if let Ok(stream) = crate::backend::rt_thread::run_with_default_output(rt_state) {
            bound_gui_state.bpm = project_save_state.backend.tempo_map.bpm();
            update_gui();

            // TODO: Loop state

            for (timeline_track_index, timeline_track) in
                project_save_state.backend.timeline_tracks.iter().enumerate()
            {
                if let Err(mut load_errors) = backend_handle
                    .add_timeline_track(timeline_track.clone(), &mut self.backend_save_state)
                {
                    for e in load_errors.drain(..) {
                        resource_load_errors.push(e);
                    }
                }

                // TODO: GUI stuff
            }

            self.backend_handle = Some(backend_handle);
            self.stream = Some(stream);

            bound_gui_state.backend_loaded = true;
        } else {
            // TODO: Better errors
            log::error!("Failed to start audio stream");
            // TODO: Remove this panic
            panic!("Failed to start audio stream");
        }
    }
}
