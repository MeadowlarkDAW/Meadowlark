

use tuix::*;

use crate::backend::ProjectStateInterface;

#[derive(Debug, Clone, PartialEq)]
pub enum TempoEvent {
    SetBPM(i32),
}


#[derive(Lens)]
pub struct AppData {
    project_interface: ProjectStateInterface,

    beats_per_minute: i32,
}


impl AppData {
    pub fn new(project_interface: ProjectStateInterface) -> Self {
        Self { 
            project_interface,
            beats_per_minute: 130,
        }
    }
}

impl Model for AppData {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(tempo_event) = event.message.downcast() {
            match tempo_event {
                TempoEvent::SetBPM(value) => {

                    // This is where we would call into the backend using self.project_interface

                    self.beats_per_minute = *value;
                    entity.emit(state, BindEvent::Update);

                    self.project_interface.set_bpm(*value as f64);
                }
            }
        }
    }
}