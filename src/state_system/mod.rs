use vizia::prelude::*;

pub mod actions;
pub mod browser_panel_state;
pub mod engine_handle;

pub use actions::{AppAction, BrowserPanelAction};
pub use browser_panel_state::{BrowserListEntryType, BrowserPanelState, BrowserPanelTab};

use self::engine_handle::EngineHandle;

#[derive(Lens)]
pub struct StateSystem {
    pub browser_panel_state: BrowserPanelState,

    #[lens(ignore)]
    pub engine_handle: EngineHandle,
}

impl StateSystem {
    pub fn new() -> Self {
        let browser_panel_state = BrowserPanelState::new();

        let engine_handle = EngineHandle::new(&browser_panel_state);

        Self { browser_panel_state, engine_handle }
    }
}

impl Model for StateSystem {
    // Update the program layer here
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_action, _| match app_action {
            AppAction::PollEngine => {
                self.engine_handle.poll_engine();
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
                    let volume_normalized = volume_normalized.clamp(0.0, 1.0);
                    self.browser_panel_state.volume_normalized = volume_normalized;

                    if let Some(activated_state) = &mut self.engine_handle.activated_state {
                        self.engine_handle
                            .ds_engine
                            .plugin_host_mut(&activated_state.sample_browser_plug_id)
                            .unwrap()
                            .set_param_value(
                                activated_state.sample_browser_plug_params[0],
                                f64::from(volume_normalized),
                            )
                            .unwrap();
                    }
                }
                BrowserPanelAction::SelectEntryByIndex(index) => {
                    self.browser_panel_state.select_entry_by_index(*index, &mut self.engine_handle);
                }
                BrowserPanelAction::EnterParentDirectory => {
                    self.browser_panel_state.enter_parent_directory();
                }
                BrowserPanelAction::EnterRootDirectory => {
                    self.browser_panel_state.enter_root_directory();
                }
                BrowserPanelAction::SetPlaybackOnSelect(val) => {
                    self.browser_panel_state.playback_on_select = *val;
                }
                BrowserPanelAction::StopPlayback => {
                    if let Some(activated_state) = &mut self.engine_handle.activated_state {
                        activated_state.sample_browser_plug_handle.stop();
                    }
                }
                BrowserPanelAction::Refresh => {
                    self.browser_panel_state.refresh();
                }
            },
        });
    }
}
