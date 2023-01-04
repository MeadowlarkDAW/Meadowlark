pub mod app_state;
pub mod project_state;

pub use app_state::*;
pub use project_state::*;

/// The state of the app/project which is considered the "source of truth".
///
/// All other state is derived from this "source of truth" state.
///
/// This is only allowed to be mutated within the `state_system::handle_action` method..
pub struct SourceState {
    pub app: AppState,
    pub current_project: Option<ProjectState>,
}

impl SourceState {
    pub fn test_project() -> Self {
        Self { app: AppState::new(), current_project: Some(ProjectState::test_project()) }
    }
}
