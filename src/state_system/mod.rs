use pcm_loader::ResampleQuality;
use vizia::prelude::*;

use crate::backend::engine_handle::EnginePollStatus;
use crate::backend::resource_loader::PcmKey;
use crate::backend::EngineHandle;

pub mod actions;
pub mod app_state;
pub mod bound_ui_state;

pub use actions::{AppAction, BrowserPanelAction, TrackAction};
pub use app_state::AppState;
pub use bound_ui_state::BoundUiState;

#[derive(Lens)]
pub struct StateSystem {
    #[lens(ignore)]
    pub app_state: AppState,

    #[lens(ignore)]
    pub engine_handle: EngineHandle,

    pub bound_ui_state: BoundUiState,
}

impl StateSystem {
    pub fn new() -> Self {
        let app_state = AppState::new();

        let engine_handle = EngineHandle::new(&app_state);
        let bound_ui_state = BoundUiState::new(&app_state);

        Self { app_state, bound_ui_state, engine_handle }
    }
}

impl Model for StateSystem {
    // Update the program layer here
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_action, _| match app_action {
            AppAction::PollEngine => match self.engine_handle.poll_engine() {
                EnginePollStatus::Ok => {}
                EnginePollStatus::EngineDeactivatedGracefully => {
                    log::info!("Engine deactivated gracefully");
                }
                EnginePollStatus::EngineCrashed(error_msg) => {
                    log::error!("Engine crashed: {}", error_msg);
                }
            },
            AppAction::BrowserPanel(action) => match action {
                BrowserPanelAction::SetPanelShown(shown) => {
                    self.app_state.browser_panel.panel_shown = *shown;
                    self.bound_ui_state.browser_panel.panel_shown = *shown;
                }
                BrowserPanelAction::SelectTab(tab) => {
                    self.app_state.browser_panel.current_tab = *tab;
                    self.bound_ui_state.browser_panel.current_tab = *tab;
                }
                BrowserPanelAction::SetPanelWidth(width) => {
                    self.app_state.browser_panel.panel_width = width.clamp(170.0, 2000.0);
                    self.bound_ui_state.browser_panel.panel_width =
                        self.app_state.browser_panel.panel_width;
                }
                BrowserPanelAction::SetSearchText(text) => {
                    self.bound_ui_state.browser_panel.search_text = text.clone();
                }
                BrowserPanelAction::SetVolumeNormalized(volume_normalized) => {
                    let volume_normalized = volume_normalized.clamp(0.0, 1.0);
                    self.app_state.browser_panel.volume_normalized = volume_normalized;
                    self.bound_ui_state.browser_panel.volume_normalized = volume_normalized;

                    if let Some(activated_handles) = &mut self.engine_handle.activated_handles {
                        self.engine_handle
                            .ds_engine
                            .plugin_host_mut(&activated_handles.sample_browser_plug_id)
                            .unwrap()
                            .set_param_value(
                                activated_handles.sample_browser_plug_params[0],
                                f64::from(volume_normalized),
                            )
                            .unwrap();
                    }
                }
                BrowserPanelAction::SelectEntryByIndex { index, invoked_by_play_btn } => {
                    self.bound_ui_state.browser_panel.select_entry_by_index(
                        cx,
                        *index,
                        *invoked_by_play_btn,
                    );
                }
                BrowserPanelAction::EnterParentDirectory => {
                    self.bound_ui_state.browser_panel.enter_parent_directory();
                }
                BrowserPanelAction::EnterRootDirectory => {
                    self.bound_ui_state.browser_panel.enter_root_directory();
                }
                BrowserPanelAction::SetPlaybackOnSelect(val) => {
                    self.app_state.browser_panel.playback_on_select = *val;
                    self.bound_ui_state.browser_panel.playback_on_select = *val;
                }
                BrowserPanelAction::PlayFile(path) => {
                    if let Some(activated_handles) = &mut self.engine_handle.activated_handles {
                        let pcm_key = PcmKey {
                            path: path.clone(),
                            resample_to_project_sr: true,
                            resample_quality: ResampleQuality::Linear,
                        };
                        match activated_handles.resource_loader.try_load(&pcm_key) {
                            Ok(pcm) => {
                                activated_handles.sample_browser_plug_handle.play_pcm(pcm);
                            }
                            Err(e) => log::error!("{}", e),
                        }
                    }
                }
                BrowserPanelAction::StopPlayback => {
                    if let Some(activated_handles) = &mut self.engine_handle.activated_handles {
                        activated_handles.sample_browser_plug_handle.stop();
                    }
                }
                BrowserPanelAction::Refresh => {
                    self.bound_ui_state.browser_panel.refresh();
                }
            },
            AppAction::Track(action) => match action {
                TrackAction::ResizeMasterTrackLane { height } => {
                    let height = height.clamp(30.0, 2000.0);

                    self.app_state.tracks_state.master_track_lane_height = height;
                    self.bound_ui_state.master_track_header.height = height;
                }
                TrackAction::ResizeTrackLaneByIndex { index, height } => {
                    let height = height.clamp(30.0, 2000.0);

                    if let Some(track_header_state) =
                        self.app_state.tracks_state.tracks.get_mut(*index)
                    {
                        track_header_state.lane_height = height;
                        self.bound_ui_state.track_headers.get_mut(*index).unwrap().height = height;
                    }
                }
            },
        });
    }
}
