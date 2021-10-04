use tuix::{Entity, Event, Lens, Model, State};

use super::{ProjectSaveState, StateSystem};

#[derive(Lens)]
pub struct BoundGuiState {
    #[lens(ignore)]
    pub state_system: Option<StateSystem>,

    pub save_state: ProjectSaveState,

    pub backend_loaded: bool,
    pub is_playing: bool,
    pub bpm: f64,
}

impl BoundGuiState {
    pub fn new() -> Self {
        Self {
            state_system: Some(StateSystem::new()),
            save_state: ProjectSaveState::new_empty(),
            backend_loaded: false,
            is_playing: false,
            bpm: 110.0,
        }
    }
}

impl Model for BoundGuiState {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {
        if let Some(state_system_event) = event.message.downcast() {
            // This is to get around the borrow checker.
            let mut state_system = self.state_system.take().unwrap();

            state_system.on_event(self, state, entity, state_system_event);

            self.state_system = Some(state_system);
        }
    }
}
