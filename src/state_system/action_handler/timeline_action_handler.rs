use dropseed::plugin_api::transport::LoopState;
use vizia::prelude::*;

use crate::state_system::{EngineHandle, SourceState, TimelineAction, WorkingState};
use crate::ui::panels::timeline_panel::{TimelineViewEvent, MAX_ZOOM, MIN_ZOOM};

pub fn handle_timeline_action(
    action: &TimelineAction,
    cx: &mut EventContext,
    source_state: &mut SourceState,
    working_state: &mut WorkingState,
    engine_handle: &mut EngineHandle,
) {
    match action {
        TimelineAction::Navigate {
            /// The horizontal zoom level. 0.25 = default zoom
            horizontal_zoom,
            /// The x position of the left side of the timeline view.
            scroll_beats_x,
        } => {
            let horizontal_zoom = horizontal_zoom.clamp(MIN_ZOOM, MAX_ZOOM);

            if let Some(project_state) = &mut source_state.project {
                project_state.timeline_horizontal_zoom = horizontal_zoom;
            }

            {
                working_state
                    .shared_timeline_view_state
                    .borrow_mut()
                    .navigate(horizontal_zoom, *scroll_beats_x);
            }
            cx.emit_to(working_state.timeline_view_id.unwrap(), TimelineViewEvent::Navigated);
        }
        TimelineAction::TransportPlay => {
            working_state.transport_playing = true;

            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                activated_handles.engine_info.transport_handle.set_playing(true);
            }

            if let Some(project_state) = &source_state.project {
                {
                    working_state.shared_timeline_view_state.borrow_mut().transport_playing = true;
                }
                cx.emit_to(
                    working_state.timeline_view_id.unwrap(),
                    TimelineViewEvent::TransportStateChanged,
                );
            }
        }
        TimelineAction::TransportPause => {
            working_state.transport_playing = false;

            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                activated_handles.engine_info.transport_handle.set_playing(false);
            }

            if let Some(project_state) = &source_state.project {
                {
                    let mut timeline_state = working_state.shared_timeline_view_state.borrow_mut();

                    timeline_state.transport_playing = false;
                    timeline_state.use_current_playhead_as_seek_pos();
                }
                cx.emit_to(
                    working_state.timeline_view_id.unwrap(),
                    TimelineViewEvent::TransportStateChanged,
                );
            }
        }
        TimelineAction::TransportStop => {
            working_state.transport_playing = false;

            if let Some(project_state) = &source_state.project {
                if let Some(activated_handles) = &mut engine_handle.activated_handles {
                    activated_handles.engine_info.transport_handle.set_playing(false);

                    let frame = project_state
                        .tempo_map
                        .timestamp_to_nearest_frame_round(project_state.playhead_last_seeked);
                    activated_handles.engine_info.transport_handle.seek_to_frame(frame.0);
                }

                {
                    let mut timeline_state = working_state.shared_timeline_view_state.borrow_mut();

                    timeline_state.transport_playing = false;
                    timeline_state.set_playhead_seek_pos(project_state.playhead_last_seeked);
                }
                cx.emit_to(
                    working_state.timeline_view_id.unwrap(),
                    TimelineViewEvent::TransportStateChanged,
                );
            }
        }
        TimelineAction::SetLoopActive(loop_active) => {
            if let Some(project_state) = &mut source_state.project {
                project_state.loop_active = *loop_active;
                working_state.transport_loop_active = *loop_active;

                if let Some(activated_handles) = &mut engine_handle.activated_handles {
                    let loop_state = if *loop_active {
                        LoopState::Active {
                            loop_start_frame: project_state
                                .tempo_map
                                .timestamp_to_nearest_frame_round(project_state.loop_start)
                                .0,
                            loop_end_frame: project_state
                                .tempo_map
                                .timestamp_to_nearest_frame_round(project_state.loop_end)
                                .0,
                        }
                    } else {
                        LoopState::Inactive
                    };

                    activated_handles.engine_info.transport_handle.set_loop_state(loop_state)
                }

                {
                    working_state.shared_timeline_view_state.borrow_mut().loop_active =
                        *loop_active;
                }
                cx.emit_to(
                    working_state.timeline_view_id.unwrap(),
                    TimelineViewEvent::TransportStateChanged,
                );
            }
        }
        TimelineAction::SelectTool(t) => {
            source_state.app.selected_timeline_tool = *t;
            working_state.selected_timeline_tool = *t;

            {
                working_state.shared_timeline_view_state.borrow_mut().selected_tool = *t;
            }
            cx.emit_to(working_state.timeline_view_id.unwrap(), TimelineViewEvent::ToolsChanged);
        }
        TimelineAction::SetSnapActive(snap) => {
            source_state.app.timeline_snap_active = *snap;
            working_state.timeline_snap_active = *snap;

            {
                working_state.shared_timeline_view_state.borrow_mut().snap_active = *snap;
            }
            cx.emit_to(working_state.timeline_view_id.unwrap(), TimelineViewEvent::ToolsChanged);
        }
        TimelineAction::SetSnapMode(mode) => {
            source_state.app.timeline_snap_mode = *mode;
            working_state.timeline_snap_mode = *mode;

            {
                working_state.shared_timeline_view_state.borrow_mut().snap_mode = *mode;
            }
            cx.emit_to(working_state.timeline_view_id.unwrap(), TimelineViewEvent::ToolsChanged);
        }
        TimelineAction::ZoomIn => {
            // TODO
        }
        TimelineAction::ZoomOut => {
            // TODO
        }
        TimelineAction::ZoomReset => {
            // TODO
        }
        TimelineAction::SelectSingleClip { track_index, clip_index } => {
            {
                working_state
                    .shared_timeline_view_state
                    .borrow_mut()
                    .select_single_clip(*track_index, *clip_index);
            }
            cx.emit_to(
                working_state.timeline_view_id.unwrap(),
                TimelineViewEvent::ClipSelectionChanged,
            );
        }
        TimelineAction::DeselectAllClips => {
            {
                working_state.shared_timeline_view_state.borrow_mut().deselect_all_clips();
            }
            cx.emit_to(
                working_state.timeline_view_id.unwrap(),
                TimelineViewEvent::ClipSelectionChanged,
            );
        }
        TimelineAction::SetClipStartPosition { track_index, clip_index, timeline_start } => {
            if let Some(project_state) = &mut source_state.project {
                if let Some(track_state) = project_state.tracks.get_mut(*track_index) {
                    if let Some(clip_state) = track_state.clips.get_mut(*clip_index) {
                        {
                            clip_state.borrow_mut().timeline_start = *timeline_start;
                        }

                        if let Some(activated_handles) = &mut engine_handle.activated_handles {}

                        {
                            working_state.shared_timeline_view_state.borrow_mut().sync_clip(
                                *track_index,
                                *clip_index,
                                &project_state.tempo_map,
                            );
                        }
                        cx.emit_to(
                            working_state.timeline_view_id.unwrap(),
                            TimelineViewEvent::ClipStateChanged {
                                track_index: *track_index,
                                clip_index: *clip_index,
                            },
                        );
                    }
                }
            }
        }
    }
}
