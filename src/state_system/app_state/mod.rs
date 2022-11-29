pub mod browser_panel_state;
pub mod palette;
pub mod tracks_state;

pub use browser_panel_state::{BrowserPanelState, BrowserPanelTab};
pub use palette::PaletteColor;
pub use tracks_state::{TrackRouteType, TrackState, TrackType, TracksState};

#[derive(Debug, Clone)]
pub struct AppState {
    pub browser_panel: BrowserPanelState,
    pub tracks_state: TracksState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            browser_panel: BrowserPanelState {
                panel_shown: true,
                current_tab: BrowserPanelTab::Samples,
                panel_width: 200.0,
                volume_normalized: 1.0,
                volume_default_normalized: 1.0,
                playback_on_select: true,
            },
            tracks_state: TracksState::new(),
        }
    }
}
