use vizia::prelude::*;

pub mod actions;
pub mod bound_ui_state;

pub use actions::Action;
pub use bound_ui_state::BoundUiState;

#[derive(Lens)]
pub struct StateSystem {
    bound_ui_state: BoundUiState,
}

impl StateSystem {
    pub fn new() -> Self {
        Self { bound_ui_state: BoundUiState::new() }
    }

    fn poll_engine(&mut self) {}
}

impl Model for StateSystem {
    // Update the program layer here
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|action, _| match action {
            Action::PollEngine => {
                self.poll_engine();
            }
        });
    }
}
