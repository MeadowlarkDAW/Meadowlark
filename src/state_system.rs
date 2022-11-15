use vizia::prelude::*;

pub mod actions;
pub mod bound_ui_state;

pub use actions::AppAction;
pub use bound_ui_state::{BoundUiState, BrowserPanelTab};

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
        event.map(|app_action, _| match app_action {
            AppAction::PollEngine => {
                self.poll_engine();
            }
            AppAction::ToggleBrowserPanelShown => {
                self.bound_ui_state.browser_panel_shown = !self.bound_ui_state.browser_panel_shown;
            }
            AppAction::SelectBrowserPanelTab(tab) => {
                self.bound_ui_state.browser_panel_tab = *tab;
            }
            AppAction::SetBrowserPanelWidth(width) => {
                self.bound_ui_state.browser_panel_width = width.clamp(150.0, 500.0);
            }
            AppAction::SetBrowserPanelSearchText(text) => {
                self.bound_ui_state.browser_panel_search_text = text.clone();
            }
            AppAction::SetBrowserVolumeNormalized(volume_normalized) => {
                self.bound_ui_state.browser_panel_volume_normalized = *volume_normalized;
            }
        });
    }
}
