use vizia::prelude::*;

use crate::ui::panels::browser_panel::BoundBrowserPanelState;
use crate::ui::panels::timeline_panel::track_headers_panel::BoundTrackHeadersPanelState;

use super::AppState;

#[derive(Lens)]
pub struct BoundUiState {
    pub browser_panel: BoundBrowserPanelState,
    pub track_headers_panel: BoundTrackHeadersPanelState,
}

impl BoundUiState {
    pub fn new(app_state: &AppState) -> Self {
        Self {
            browser_panel: BoundBrowserPanelState::new(&app_state),
            track_headers_panel: BoundTrackHeadersPanelState::new(&app_state),
        }
    }
}
