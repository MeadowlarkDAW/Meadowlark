#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineMode {
    /// In this mode, the timeline displays content in units of measures,
    /// bars, beats, and sub-beats.
    Musical,
    /// In this mode, the timeline displays content in units of hours,
    /// minutes, seconds, and milliseconds.
    HMS,
}

#[derive(Debug, Clone)]
pub struct TimelineState {
    /// The horizontal zoom level. 1.0 = default zoom
    pub horizontal_zoom: f64,

    /// The x position of the left side of the view. When the timeline is in
    /// musical mode, this is in units of beats. When the timeline is in
    /// H:M:S mode, this is in units of seconds.
    pub scroll_units_x: f64,

    /// The mode in which the timeline displays its contents.
    pub mode: TimelineMode,
}

impl TimelineState {
    pub fn new() -> Self {
        Self { horizontal_zoom: 1.0, scroll_units_x: 0.0, mode: TimelineMode::Musical }
    }
}
