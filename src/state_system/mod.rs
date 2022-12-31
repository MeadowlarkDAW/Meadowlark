use pcm_loader::ResampleQuality;
use std::cell::RefCell;
use std::rc::Rc;
use vizia::prelude::*;

use crate::backend::resource_loader::PcmKey;
use crate::backend::EngineHandle;
use crate::{
    backend::engine_handle::EnginePollStatus, ui::panels::timeline_panel::TimelineViewState,
};

pub mod actions;
pub mod derived_state;
pub mod source_state;
pub mod time;

pub use actions::{Action, BrowserPanelAction, ScrollUnits, TrackAction};
pub use derived_state::DerivedState;
pub use source_state::SourceState;

use crate::ui::panels::timeline_panel::{
    track_header_view::MIN_TRACK_HEADER_HEIGHT, TimelineViewEvent, MAX_ZOOM, MIN_ZOOM,
};

use self::actions::{InternalAction, TimelineAction};

/// The `StateSystem` struct is in charge of listening to `Action`s sent from sources
/// such as UI views and scripts, and then mutating state and manipulating the backend
/// accordingly.
///
/// No other struct is allowed to mutate this state or manipulate the backend. They
/// must send `Action`s to this struct to achieve this.
///
/// State is divided into two parts: the `SourceState` and the `DerivedState`.
/// * The `SourceState` contains all state in the app/project which serves as
/// the "source of truth" that all other state is derived from. This can be thought of
/// as the state that gets saved to disk when saving a project or a config file.
/// * The `DerivedState` contains all the working state of the application. This
/// includes things like lenses to UI elements, as well as cached data for the
/// position of elements in the timeline view.
#[derive(Lens)]
pub struct StateSystem {
    #[lens(ignore)]
    pub source_state: SourceState,

    #[lens(ignore)]
    pub engine_handle: EngineHandle,

    pub derived_state: DerivedState,
}

impl StateSystem {
    pub fn new(shared_timeline_view_state: Rc<RefCell<TimelineViewState>>) -> Self {
        let source_state = SourceState::test_project();

        let engine_handle = EngineHandle::new(&source_state);
        let derived_state = DerivedState::new(&source_state, shared_timeline_view_state);

        Self { source_state, derived_state, engine_handle }
    }
}

impl Model for StateSystem {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_action, _| match app_action {
            Action::PollEngine => match self.engine_handle.poll_engine() {
                EnginePollStatus::Ok => {}
                EnginePollStatus::EngineDeactivatedGracefully => {
                    log::info!("Engine deactivated gracefully");
                }
                EnginePollStatus::EngineCrashed(error_msg) => {
                    log::error!("Engine crashed: {}", error_msg);
                }
            },
            Action::BrowserPanel(action) => match action {
                BrowserPanelAction::SetPanelShown(shown) => {
                    self.source_state.app.browser_panel.panel_shown = *shown;
                    self.derived_state.browser_panel_lens.panel_shown = *shown;
                }
                BrowserPanelAction::SelectTab(tab) => {
                    self.source_state.app.browser_panel.current_tab = *tab;
                    self.derived_state.browser_panel_lens.current_tab = *tab;
                }
                BrowserPanelAction::SetPanelWidth(width) => {
                    let width = width.clamp(170.0, 2000.0);
                    self.source_state.app.browser_panel.panel_width = width;
                    self.derived_state.browser_panel_lens.panel_width = width;
                }
                BrowserPanelAction::SetSearchText(text) => {
                    self.derived_state.browser_panel_lens.search_text = text.clone();
                }
                BrowserPanelAction::SetVolumeNormalized(volume_normalized) => {
                    let volume_normalized = volume_normalized.clamp(0.0, 1.0);

                    self.source_state.app.browser_panel.volume_normalized = volume_normalized;
                    self.derived_state.browser_panel_lens.volume.value_normalized =
                        volume_normalized;

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
                    self.derived_state.browser_panel_lens.select_entry_by_index(
                        cx,
                        *index,
                        *invoked_by_play_btn,
                    );
                }
                BrowserPanelAction::EnterParentDirectory => {
                    self.derived_state.browser_panel_lens.enter_parent_directory();
                }
                BrowserPanelAction::EnterRootDirectory => {
                    self.derived_state.browser_panel_lens.enter_root_directory();
                }
                BrowserPanelAction::SetPlaybackOnSelect(val) => {
                    self.source_state.app.browser_panel.playback_on_select = *val;
                    self.derived_state.browser_panel_lens.playback_on_select = *val;
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
                    self.derived_state.browser_panel_lens.refresh();
                }
            },
            Action::Track(action) => match action {
                TrackAction::SelectMasterTrack => {
                    self.derived_state.track_headers_panel_lens.select_master_track();
                }
                TrackAction::SelectTrack { index } => {
                    self.derived_state.track_headers_panel_lens.select_track_by_index(*index);
                }
                TrackAction::SetMasterTrackVolumeNormalized(volume_normalized) => {
                    if let Some(project_state) = &mut self.source_state.current_project {
                        let volume_normalized = volume_normalized.clamp(0.0, 1.0);

                        project_state.master_track_volume_normalized = volume_normalized;
                        self.derived_state
                            .track_headers_panel_lens
                            .master_track_header
                            .volume
                            .value_normalized = volume_normalized;
                    }
                }
                TrackAction::SetMasterTrackPanNormalized(pan_normalized) => {
                    if let Some(project_state) = &mut self.source_state.current_project {
                        let pan_normalized = pan_normalized.clamp(0.0, 1.0);

                        project_state.master_track_pan_normalized = pan_normalized;
                        self.derived_state
                            .track_headers_panel_lens
                            .master_track_header
                            .pan
                            .value_normalized = pan_normalized;
                    }
                }
                TrackAction::SetMasterTrackHeight { height } => {
                    if let Some(project_state) = &mut self.source_state.current_project {
                        let height = height.clamp(MIN_TRACK_HEADER_HEIGHT, 2000.0);

                        project_state.master_track_lane_height = height;
                        self.derived_state.track_headers_panel_lens.master_track_header.height =
                            height;
                    }
                }
                TrackAction::SetTrackHeight { index, height } => {
                    if let Some(project_state) = &mut self.source_state.current_project {
                        let height = height.clamp(MIN_TRACK_HEADER_HEIGHT, 2000.0);

                        let is_some = if let Some(track_header_state) =
                            project_state.tracks.get_mut(*index)
                        {
                            track_header_state.lane_height = height;
                            self.derived_state
                                .track_headers_panel_lens
                                .track_headers
                                .get_mut(*index)
                                .unwrap()
                                .height = height;

                            true
                        } else {
                            false
                        };
                        if is_some {
                            {
                                self.derived_state
                                    .shared_timeline_view_state
                                    .borrow_mut()
                                    .set_track_height(*index, height);
                            }
                            cx.emit_to(
                                self.derived_state.timeline_view_id.unwrap(),
                                TimelineViewEvent::TrackHeightSet { index: *index },
                            );
                        }
                    }
                }
                TrackAction::SetTrackVolumeNormalized { index, volume_normalized } => {
                    if let Some(project_state) = &mut self.source_state.current_project {
                        let volume_normalized = volume_normalized.clamp(0.0, 1.0);

                        if let Some(track_header_state) = project_state.tracks.get_mut(*index) {
                            track_header_state.volume_normalized = volume_normalized;
                            self.derived_state
                                .track_headers_panel_lens
                                .track_headers
                                .get_mut(*index)
                                .unwrap()
                                .volume
                                .value_normalized = volume_normalized;
                        }
                    }
                }
                TrackAction::SetTrackPanNormalized { index, pan_normalized } => {
                    if let Some(project_state) = &mut self.source_state.current_project {
                        let pan_normalized = pan_normalized.clamp(0.0, 1.0);

                        if let Some(track_header_state) = project_state.tracks.get_mut(*index) {
                            track_header_state.pan_normalized = pan_normalized;
                            self.derived_state
                                .track_headers_panel_lens
                                .track_headers
                                .get_mut(*index)
                                .unwrap()
                                .pan
                                .value_normalized = pan_normalized;
                        }
                    }
                }
            },
            Action::Timeline(action) => match action {
                TimelineAction::Navigate {
                    /// The horizontal zoom level. 0.25 = default zoom
                    horizontal_zoom,
                    /// The x position of the left side of the timeline view.
                    scroll_units_x,
                } => {
                    let horizontal_zoom = horizontal_zoom.clamp(MIN_ZOOM, MAX_ZOOM);

                    if let Some(project_state) = &mut self.source_state.current_project {
                        project_state.timeline_horizontal_zoom = horizontal_zoom;
                    }

                    {
                        self.derived_state
                            .shared_timeline_view_state
                            .borrow_mut()
                            .navigate(horizontal_zoom, *scroll_units_x);
                    }
                    cx.emit_to(
                        self.derived_state.timeline_view_id.unwrap(),
                        TimelineViewEvent::Navigated,
                    );
                }
                TimelineAction::TransportPlay => {
                    self.derived_state.transport_playing = true;

                    if let Some(activated_handles) = &mut self.engine_handle.activated_handles {
                        activated_handles.engine_info.transport_handle.set_playing(true);
                    }
                }
                TimelineAction::TransportPause => {
                    self.derived_state.transport_playing = false;

                    if let Some(activated_handles) = &mut self.engine_handle.activated_handles {
                        activated_handles.engine_info.transport_handle.set_playing(false);
                    }
                }
                TimelineAction::TransportStop => {
                    self.derived_state.transport_playing = false;

                    if let Some(activated_handles) = &mut self.engine_handle.activated_handles {
                        activated_handles.engine_info.transport_handle.set_playing(false);

                        // TODO: Seek to last-seeked position instead of the beginning.
                        activated_handles.engine_info.transport_handle.seek_to_frame(0);
                    }
                }
            },
            Action::_Internal(action) => match action {
                InternalAction::TimelineViewID(id) => {
                    if self.derived_state.timeline_view_id.is_none() {
                        self.derived_state.timeline_view_id = Some(*id);

                        if let Some(project_state) = &self.source_state.current_project {
                            {
                                self.derived_state
                                    .shared_timeline_view_state
                                    .borrow_mut()
                                    .sync_from_project_state(project_state);
                            }
                            cx.emit_to(
                                self.derived_state.timeline_view_id.unwrap(),
                                TimelineViewEvent::SyncedFromProjectState,
                            );
                        }
                    }
                }
            },
        });
    }
}
