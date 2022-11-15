use vizia::prelude::*;

pub mod actions;
pub mod bound_ui_state;

pub use actions::AppAction;
pub use bound_ui_state::{BoundUiState, BrowserPanelTab};

use crate::state_system::bound_ui_state::BrowserListEntryType;

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
            AppAction::BrowserItemSelected(index) => {
                if let Some(old_entry_i) = self.bound_ui_state.selected_browser_entry.take() {
                    if let Some(old_entry) =
                        &mut self.bound_ui_state.browser_list_entries.get_mut(old_entry_i)
                    {
                        old_entry.selected = false;
                    }
                }

                if let Some(entry) = self.bound_ui_state.browser_list_entries.get_mut(*index) {
                    match entry.type_ {
                        BrowserListEntryType::AudioFile => {
                            self.bound_ui_state.selected_browser_entry = Some(*index);
                            entry.selected = true;
                        }
                        BrowserListEntryType::Folder => {}
                    }
                }
            }
        });
    }
}
