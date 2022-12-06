use pcm_loader::ResampleQuality;
use vizia::prelude::*;

use crate::backend::engine_handle::EnginePollStatus;
use crate::backend::resource_loader::PcmKey;
use crate::backend::EngineHandle;

pub mod actions;
pub mod app_state;
pub mod bound_ui_state;

pub use actions::{AppAction, BrowserPanelAction, ScrollUnits, TrackAction};
pub use app_state::AppState;
pub use bound_ui_state::BoundUiState;

use crate::ui::panels::timeline_panel::{
    track_header_view::MIN_TRACK_HEADER_HEIGHT, TimelineViewEvent, MAX_ZOOM, MIN_ZOOM,
};

use self::actions::{InternalAction, TimelineAction};

#[derive(Lens)]
pub struct StateSystem {
    #[lens(ignore)]
    pub app_state: AppState,

    #[lens(ignore)]
    pub engine_handle: EngineHandle,

    #[lens(ignore)]
    pub timeline_view_id: Option<Entity>,

    pub bound_ui_state: BoundUiState,
}

impl StateSystem {
    pub fn new(app_state: AppState) -> Self {
        let engine_handle = EngineHandle::new(&app_state);
        let bound_ui_state = BoundUiState::new(&app_state);

        Self { app_state, bound_ui_state, timeline_view_id: None, engine_handle }
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
                    self.bound_ui_state.browser_panel.volume.value_normalized = volume_normalized;

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
                TrackAction::SelectMasterTrack => {
                    self.bound_ui_state.track_headers_panel.select_master_track();
                }
                TrackAction::SelectTrack { index } => {
                    self.bound_ui_state.track_headers_panel.select_track_by_index(*index);
                }
                TrackAction::SetMasterTrackVolumeNormalized(volume_normalized) => {
                    let volume_normalized = volume_normalized.clamp(0.0, 1.0);
                    self.app_state.tracks_state.master_track_volume_normalized = volume_normalized;
                    self.bound_ui_state
                        .track_headers_panel
                        .master_track_header
                        .volume
                        .value_normalized = volume_normalized;
                }
                TrackAction::SetMasterTrackPanNormalized(pan_normalized) => {
                    let pan_normalized = pan_normalized.clamp(0.0, 1.0);
                    self.app_state.tracks_state.master_track_pan_normalized = pan_normalized;
                    self.bound_ui_state
                        .track_headers_panel
                        .master_track_header
                        .pan
                        .value_normalized = pan_normalized;
                }
                TrackAction::ResizeMasterTrackLane { height } => {
                    let height = height.clamp(MIN_TRACK_HEADER_HEIGHT, 2000.0);

                    self.app_state.tracks_state.master_track_lane_height = height;
                    self.bound_ui_state.track_headers_panel.master_track_header.height = height;
                }
                TrackAction::ResizeTrackLane { index, height } => {
                    let height = height.clamp(MIN_TRACK_HEADER_HEIGHT, 2000.0);

                    if let Some(track_header_state) =
                        self.app_state.tracks_state.tracks.get_mut(*index)
                    {
                        track_header_state.lane_height = height;
                        self.bound_ui_state
                            .track_headers_panel
                            .track_headers
                            .get_mut(*index)
                            .unwrap()
                            .height = height;
                    }
                }
                TrackAction::SetTrackVolumeNormalized { index, volume_normalized } => {
                    let volume_normalized = volume_normalized.clamp(0.0, 1.0);

                    if let Some(track_header_state) =
                        self.app_state.tracks_state.tracks.get_mut(*index)
                    {
                        track_header_state.volume_normalized = volume_normalized;
                        self.bound_ui_state
                            .track_headers_panel
                            .track_headers
                            .get_mut(*index)
                            .unwrap()
                            .volume
                            .value_normalized = volume_normalized;
                    }
                }
                TrackAction::SetTrackPanNormalized { index, pan_normalized } => {
                    let pan_normalized = pan_normalized.clamp(0.0, 1.0);

                    if let Some(track_header_state) =
                        self.app_state.tracks_state.tracks.get_mut(*index)
                    {
                        track_header_state.pan_normalized = pan_normalized;
                        self.bound_ui_state
                            .track_headers_panel
                            .track_headers
                            .get_mut(*index)
                            .unwrap()
                            .pan
                            .value_normalized = pan_normalized;
                    }
                }
            },
            AppAction::Timeline(action) => match action {
                TimelineAction::Navigate {
                    /// The horizontal zoom level. 1.0 = default zoom
                    horizontal_zoom,
                    /// The x position of the left side of the timeline view.
                    scroll_units_x,
                } => {
                    let horizontal_zoom = horizontal_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
                    let scroll_units_x = scroll_units_x.max(0.0);

                    cx.emit_to(
                        self.timeline_view_id.unwrap(),
                        TimelineViewEvent::Navigate { horizontal_zoom, scroll_units_x },
                    );
                }
            },
            AppAction::_Internal(action) => match action {
                InternalAction::TimelineViewID(id) => {
                    if self.timeline_view_id.is_none() {
                        self.timeline_view_id = Some(*id);
                    }
                }
            },
        });
    }
}
