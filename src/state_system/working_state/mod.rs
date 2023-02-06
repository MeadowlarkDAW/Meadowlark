use std::cell::RefCell;
use std::rc::Rc;
use vizia::prelude::*;

use super::source_state::{SnapMode, TimelineTool};
use super::working_state::timeline_view_state::TimelineViewState;
use super::SourceState;

pub mod browser_panel_state;
pub mod timeline_view_state;
pub mod track_headers_panel_state;

use browser_panel_state::BrowserPanelState;
use track_headers_panel_state::TrackHeadersPanelState;

/// This contains all of the temporary working state of the app.
///
/// This includes things such as the data binding lenses for UI elements
/// and cached data such as positions of clips on the timeline view.
///
/// This is only allowed to be mutated within the
/// `state_system::handle_action` method.
#[derive(Lens)]
pub struct WorkingState {
    pub browser_panel_lens: BrowserPanelState,
    pub track_headers_panel_lens: TrackHeadersPanelState,

    pub transport_playing: bool,
    pub transport_loop_active: bool,

    pub selected_timeline_tool: TimelineTool,
    pub timeline_snap_active: bool,
    pub timeline_snap_mode: SnapMode,
    pub timeline_snap_choices: Vec<SnapMode>,

    #[lens(ignore)]
    pub timeline_view_id: Option<Entity>,

    /// This is only allowed to be borrowed mutably within the
    /// `state_system::handle_action` method.
    #[lens(ignore)]
    pub shared_timeline_view_state: Rc<RefCell<TimelineViewState>>,
}

impl WorkingState {
    pub fn new(
        state: &SourceState,
        shared_timeline_view_state: Rc<RefCell<TimelineViewState>>,
    ) -> Self {
        Self {
            browser_panel_lens: BrowserPanelState::new(&state),
            track_headers_panel_lens: TrackHeadersPanelState::new(&state),
            transport_playing: false,
            transport_loop_active: state.project.as_ref().map(|p| p.loop_active).unwrap_or(false),
            selected_timeline_tool: state.app.selected_timeline_tool,
            timeline_snap_active: state.app.timeline_snap_active,
            timeline_snap_mode: state.app.timeline_snap_mode,
            timeline_snap_choices: vec![
                SnapMode::Line,
                SnapMode::Beat,
                SnapMode::HalfBeat,
                SnapMode::QuarterBeat,
                SnapMode::EigthBeat,
                SnapMode::SixteenthBeat,
                SnapMode::_32ndBeat,
                SnapMode::ThirdBeat,
            ],
            timeline_view_id: None,
            shared_timeline_view_state,
        }
    }
}
