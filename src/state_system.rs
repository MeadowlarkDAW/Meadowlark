use vizia::prelude::*;

pub mod bound_ui_state;
pub mod events;

pub use bound_ui_state::{BoundUiState, BrowserPanelTab};
pub use events::AppEvent;

#[derive(Lens)]
pub struct StateSystem {
    pub bound_ui_state: BoundUiState,
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
        event.map(|app_event, _| match app_event {
            AppEvent::PollEngine => {
                self.poll_engine();
            }
            AppEvent::ToggleBrowserPanelShown => {
                self.bound_ui_state.browser_panel_shown = !self.bound_ui_state.browser_panel_shown;
            }
            AppEvent::SelectBrowserPanelTab(tab) => {
                self.bound_ui_state.browser_panel_tab = *tab;
            }
        });
    }
}
