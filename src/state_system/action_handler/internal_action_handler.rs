use vizia::prelude::*;

use crate::state_system::{EngineHandle, InternalAction, SourceState, WorkingState};
use crate::ui::panels::timeline_panel::TimelineViewEvent;

pub fn handle_internal_action(
    action: &InternalAction,
    cx: &mut EventContext,
    source_state: &mut SourceState,
    working_state: &mut WorkingState,
    engine_handle: &mut EngineHandle,
) {
    match action {
        InternalAction::TimelineViewID(id) => {
            if working_state.timeline_view_id.is_none() {
                working_state.timeline_view_id = Some(*id);

                if let Some(project_state) = &source_state.current_project {
                    {
                        working_state
                            .shared_timeline_view_state
                            .borrow_mut()
                            .sync_from_project_state(project_state);
                    }
                    cx.emit_to(
                        working_state.timeline_view_id.unwrap(),
                        TimelineViewEvent::SyncedFromProjectState,
                    );
                }
            }
        }
    }
}
