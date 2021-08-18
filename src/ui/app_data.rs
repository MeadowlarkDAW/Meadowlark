use tuix::*;

use crate::backend::ProjectStateInterface;

#[derive(Debug, Clone, PartialEq)]
pub enum TempoEvent {
    SetBPM(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransportEvent {
    Play,
    Stop,
    Pause,
}

#[derive(Lens)]
pub struct AppData {
    project_interface: ProjectStateInterface,

    // Tempo
    beats_per_minute: f64,

    // Transport
    is_playing: bool,
}

impl AppData {
    pub fn new(project_interface: ProjectStateInterface) -> Self {
        Self {
            project_interface,
            // Tempo
            beats_per_minute: 130.0,
            // Transport
            is_playing: false,
        }
    }
}

impl Model for AppData {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        // Tmepo Events
        if let Some(tempo_event) = event.message.downcast() {
            match tempo_event {
                TempoEvent::SetBPM(value) => {
                    let value = value.clamp(0.0, 10000.0);

                    // This is where we would call into the backend using self.project_interface

                    self.beats_per_minute = value;
                    entity.emit(state, BindEvent::Update);

                    self.project_interface.set_bpm(value);
                }
            }
        }

        // Trnasport Events
        if let Some(transport_event) = event.message.downcast() {
            match transport_event {
                TransportEvent::Play => {
                    if !self.is_playing {
                        self.is_playing = true;
                    }

                    entity.emit(state, BindEvent::Update);

                    let (transport, _) = self.project_interface.timeline_transport();
                    transport.set_playing(true);
                }

                TransportEvent::Stop => {
                    if self.is_playing {
                        self.is_playing = false;
                    }

                    entity.emit(state, BindEvent::Update);

                    let (transport, save_state) = self.project_interface.timeline_transport();
                    transport.set_playing(false);
                    // TODO: have the transport struct handle this.
                    transport.seek_to(0.0.into(), save_state);
                }

                TransportEvent::Pause => {
                    if self.is_playing {
                        self.is_playing = false;
                    }

                    entity.emit(state, BindEvent::Update);

                    let (transport, _) = self.project_interface.timeline_transport();
                    transport.set_playing(false);
                }
            }
        }
    }
}
