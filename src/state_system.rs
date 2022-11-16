use vizia::prelude::*;

pub mod actions;
pub mod browser_panel_state;

pub use actions::{AppAction, BrowserPanelAction};
pub use browser_panel_state::{BrowserListEntryType, BrowserPanelState, BrowserPanelTab};

#[derive(Lens)]
pub struct StateSystem {
    pub browser_panel_state: BrowserPanelState,
}

impl StateSystem {
    pub fn new() -> Self {
        Self { browser_panel_state: BrowserPanelState::new() }
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
            AppAction::BrowserPanel(action) => match action {
                BrowserPanelAction::SetPanelShown(bool) => {
                    self.browser_panel_state.panel_shown = !self.browser_panel_state.panel_shown;
                }
                BrowserPanelAction::SelectTab(tab) => {
                    self.browser_panel_state.current_tab = *tab;
                }
                BrowserPanelAction::SetPanelWidth(width) => {
                    self.browser_panel_state.panel_width = width.clamp(150.0, 500.0);
                }
                BrowserPanelAction::SetSearchText(text) => {
                    self.browser_panel_state.search_text = text.clone();
                }
                BrowserPanelAction::SetVolumeNormalized(volume_normalized) => {
                    self.browser_panel_state.volume_normalized = *volume_normalized;
                }
                BrowserPanelAction::SelectEntryByIndex(index) => {
                    self.browser_panel_state.select_entry_by_index(*index);
                }
                BrowserPanelAction::EnterParentDirectory => {
                    self.browser_panel_state.enter_parent_directory();
                }
                BrowserPanelAction::EnterRootDirectory => {
                    self.browser_panel_state.enter_root_directory();
                }
            },
        });
    }
}
