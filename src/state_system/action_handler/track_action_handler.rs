use vizia::prelude::*;

use crate::state_system::{EngineHandle, SourceState, TrackAction, WorkingState};
use crate::ui::panels::timeline_panel::{
    track_header_view::MIN_TRACK_HEADER_HEIGHT, TimelineViewEvent,
};

pub fn handle_track_action(
    action: &TrackAction,
    cx: &mut EventContext,
    source_state: &mut SourceState,
    working_state: &mut WorkingState,
    engine_handle: &mut EngineHandle,
) {
    match action {
        TrackAction::SelectMasterTrack => {
            working_state.track_headers_panel_lens.select_master_track();
        }
        TrackAction::SelectTrack { index } => {
            working_state.track_headers_panel_lens.select_track_by_index(*index);
        }
        TrackAction::SetMasterTrackVolumeNormalized(volume_normalized) => {
            if let Some(project_state) = &mut source_state.project {
                let volume_normalized = volume_normalized.clamp(0.0, 1.0);

                project_state.master_track_volume_normalized = volume_normalized;
                working_state
                    .track_headers_panel_lens
                    .master_track_header
                    .volume
                    .value_normalized = volume_normalized;
            }
        }
        TrackAction::SetMasterTrackPanNormalized(pan_normalized) => {
            if let Some(project_state) = &mut source_state.project {
                let pan_normalized = pan_normalized.clamp(0.0, 1.0);

                project_state.master_track_pan_normalized = pan_normalized;
                working_state.track_headers_panel_lens.master_track_header.pan.value_normalized =
                    pan_normalized;
            }
        }
        TrackAction::SetMasterTrackHeight { height } => {
            if let Some(project_state) = &mut source_state.project {
                let height = height.clamp(MIN_TRACK_HEADER_HEIGHT, 2000.0);

                project_state.master_track_lane_height = height;
                working_state.track_headers_panel_lens.master_track_header.height = height;
            }
        }
        TrackAction::SetTrackHeight { index, height } => {
            if let Some(project_state) = &mut source_state.project {
                let height = height.clamp(MIN_TRACK_HEADER_HEIGHT, 2000.0);

                let is_some = if let Some(track_header_state) = project_state.tracks.get_mut(*index)
                {
                    track_header_state.lane_height = height;
                    working_state
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
                        working_state
                            .shared_timeline_view_state
                            .borrow_mut()
                            .set_track_height(*index, height);
                    }
                    cx.emit_to(
                        working_state.timeline_view_id.unwrap(),
                        TimelineViewEvent::TrackHeightSet { index: *index },
                    );
                }
            }
        }
        TrackAction::SetTrackVolumeNormalized { index, volume_normalized } => {
            if let Some(project_state) = &mut source_state.project {
                let volume_normalized = volume_normalized.clamp(0.0, 1.0);

                if let Some(track_header_state) = project_state.tracks.get_mut(*index) {
                    track_header_state.volume_normalized = volume_normalized;
                    working_state
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
            if let Some(project_state) = &mut source_state.project {
                let pan_normalized = pan_normalized.clamp(0.0, 1.0);

                if let Some(track_header_state) = project_state.tracks.get_mut(*index) {
                    track_header_state.pan_normalized = pan_normalized;
                    working_state
                        .track_headers_panel_lens
                        .track_headers
                        .get_mut(*index)
                        .unwrap()
                        .pan
                        .value_normalized = pan_normalized;
                }
            }
        }
    }
}
