use std::cell::RefCell;
use std::rc::Rc;
use vizia::prelude::*;

use crate::backend::EngineHandle;
use crate::ui::panels::timeline_panel::TimelineViewWorkingState;

mod action_handler;
pub mod actions;
pub mod source_state;
pub mod time;
pub mod working_state;

pub use actions::{AppAction, BrowserPanelAction, TimelineAction, TrackAction};
pub use source_state::SourceState;
pub use working_state::WorkingState;

use self::actions::InternalAction;

/// The `StateSystem` struct is in charge of listening to `Action`s sent from sources
/// such as UI views and scripts, and then mutating state and manipulating the backend
/// accordingly.
///
/// No other struct is allowed to mutate this state or manipulate the backend. They
/// must send `AppAction`s to this struct to achieve this.
///
/// State is divided into two parts: the `SourceState` and the `WorkingState`.
/// * The `SourceState` contains all state in the app/project which serves as
/// the "source of truth" that all other state is derived from. This can be thought of
/// as the state that gets saved to disk when saving a project or a config file.
/// * The `WorkingState` contains all the working state of the application. This
/// includes things like lenses to UI elements, as well as cached data for the
/// position of elements in the timeline view.
#[derive(Lens)]
pub struct StateSystem {
    #[lens(ignore)]
    pub source_state: SourceState,

    #[lens(ignore)]
    pub engine_handle: EngineHandle,

    pub working_state: WorkingState,
}

impl StateSystem {
    pub fn new(shared_timeline_view_state: Rc<RefCell<TimelineViewWorkingState>>) -> Self {
        let source_state = SourceState::test_project();

        let engine_handle = EngineHandle::new(&source_state);
        let working_state = WorkingState::new(&source_state, shared_timeline_view_state);

        Self { source_state, working_state, engine_handle }
    }
}

impl Model for StateSystem {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|action, _| {
            handle_action(
                action,
                cx,
                &mut self.source_state,
                &mut self.working_state,
                &mut self.engine_handle,
            )
        });
    }
}

pub fn handle_action(
    action: &AppAction,
    cx: &mut EventContext,
    source_state: &mut SourceState,
    working_state: &mut WorkingState,
    engine_handle: &mut EngineHandle,
) {
    match action {
        AppAction::_PollEngine => {
            action_handler::poll_engine(cx, source_state, working_state, engine_handle);
        }
        AppAction::BrowserPanel(action) => {
            action_handler::handle_browser_panel_action(
                action,
                cx,
                source_state,
                working_state,
                engine_handle,
            );
        }
        AppAction::Track(action) => {
            action_handler::handle_track_action(
                action,
                cx,
                source_state,
                working_state,
                engine_handle,
            );
        }
        AppAction::Timeline(action) => {
            action_handler::handle_timeline_action(
                action,
                cx,
                source_state,
                working_state,
                engine_handle,
            );
        }
        AppAction::_Internal(action) => {
            action_handler::handle_internal_action(
                action,
                cx,
                source_state,
                working_state,
                engine_handle,
            );
        }
    }
}
