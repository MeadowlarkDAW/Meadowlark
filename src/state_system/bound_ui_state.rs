use vizia::prelude::*;

use crate::ui::panels::browser_panel::BoundBrowserPanelState;
use crate::ui::panels::timeline_panel::track_header_view::{self, BoundTrackHeaderState};

use super::AppState;

#[derive(Lens)]
pub struct BoundUiState {
    pub browser_panel: BoundBrowserPanelState,
    pub master_track_header: BoundTrackHeaderState,
    pub track_headers: Vec<BoundTrackHeaderState>,
}

impl BoundUiState {
    pub fn new(app_state: &AppState) -> Self {
        let (master_track_header, track_headers) =
            track_header_view::bound_state_from_tracks_state(&app_state.tracks_state);

        Self {
            browser_panel: BoundBrowserPanelState::new(&app_state.browser_panel),
            master_track_header,
            track_headers,
        }
    }
}
