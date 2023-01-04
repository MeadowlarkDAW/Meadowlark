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
            scroll_units_x,
        } => {
            let horizontal_zoom = horizontal_zoom.clamp(MIN_ZOOM, MAX_ZOOM);

            if let Some(project_state) = &mut source_state.current_project {
                project_state.timeline_horizontal_zoom = horizontal_zoom;
            }

            {
                working_state
                    .shared_timeline_view_state
                    .borrow_mut()
                    .navigate(horizontal_zoom, *scroll_units_x);
            }
            cx.emit_to(working_state.timeline_view_id.unwrap(), TimelineViewEvent::Navigated);
        }
        TimelineAction::TransportPlay => {
            working_state.transport_playing = true;

            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                activated_handles.engine_info.transport_handle.set_playing(true);
            }

            if let Some(project_state) = &source_state.current_project {
                {
                    working_state
                        .shared_timeline_view_state
                        .borrow_mut()
                        .set_transport_playing(true);
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

            if let Some(project_state) = &source_state.current_project {
                {
                    let mut timeline_state = working_state.shared_timeline_view_state.borrow_mut();

                    timeline_state.set_transport_playing(false);
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

            if let Some(activated_handles) = &mut engine_handle.activated_handles {
                activated_handles.engine_info.transport_handle.set_playing(false);

                // TODO: Seek to last-seeked position instead of the beginning.
                activated_handles.engine_info.transport_handle.seek_to_frame(0);
            }

            if let Some(project_state) = &source_state.current_project {
                {
                    let mut timeline_state = working_state.shared_timeline_view_state.borrow_mut();

                    timeline_state.set_transport_playing(false);
                    timeline_state.set_playhead_seek_pos(project_state.playhead_last_seeked);
                }
                cx.emit_to(
                    working_state.timeline_view_id.unwrap(),
                    TimelineViewEvent::TransportStateChanged,
                );
            }
        }
    }
}
