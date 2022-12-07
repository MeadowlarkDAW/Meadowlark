use std::cell::RefCell;
use std::rc::Rc;

pub mod browser_panel_state;
pub mod palette;
pub mod timeline_state;
pub mod tracks_state;

pub use browser_panel_state::{BrowserPanelState, BrowserPanelTab};
pub use palette::PaletteColor;
pub use timeline_state::TimelineState;
pub use tracks_state::{TrackRouteType, TrackState, TrackType, TracksState};

#[derive(Debug, Clone)]
pub struct AppState {
    pub browser_panel: BrowserPanelState,
    pub tracks_state: TracksState,
    /// Only the `StateSystem` struct is allowed to mutate this.
    pub timeline_state: Rc<RefCell<TimelineState>>,
}

impl AppState {
    pub fn new() -> Self {
        let tracks_state = TracksState::new();
        let timeline_state = TimelineState::new(&tracks_state);

        Self {
            browser_panel: BrowserPanelState {
                panel_shown: true,
                current_tab: BrowserPanelTab::Samples,
                panel_width: 200.0,
                volume_normalized: 1.0,
                volume_default_normalized: 1.0,
                playback_on_select: true,
            },
            tracks_state,
            timeline_state: Rc::new(RefCell::new(timeline_state)),
        }
    }
}
