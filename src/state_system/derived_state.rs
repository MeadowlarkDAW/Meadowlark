use std::cell::RefCell;
use std::rc::Rc;
use vizia::prelude::*;

use crate::ui::panels::browser_panel::BrowserPanelLens;
use crate::ui::panels::timeline_panel::track_headers_panel::TrackHeadersPanelLens;
use crate::ui::panels::timeline_panel::TimelineViewState;

use super::SourceState;

/// This contains all of the temporary working state of the app.
///
/// This includes things such as the data binding lenses for UI elements
/// and cached data such as positions of clips on the timeline view.
///
/// Only the `StateSystem` struct is allowed to mutate this.
#[derive(Lens)]
pub struct DerivedState {
    pub browser_panel_lens: BrowserPanelLens,
    pub track_headers_panel_lens: TrackHeadersPanelLens,

    #[lens(ignore)]
    pub timeline_view_id: Option<Entity>,

    /// Only the `StateSystem` struct is allowed to borrow this mutably.
    #[lens(ignore)]
    pub shared_timeline_view_state: Rc<RefCell<TimelineViewState>>,
}

impl DerivedState {
    pub fn new(
        state: &SourceState,
        shared_timeline_view_state: Rc<RefCell<TimelineViewState>>,
    ) -> Self {
        Self {
            browser_panel_lens: BrowserPanelLens::new(&state),
            track_headers_panel_lens: TrackHeadersPanelLens::new(&state),
            timeline_view_id: None,
            shared_timeline_view_state,
        }
    }
}
