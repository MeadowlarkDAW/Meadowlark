use rusty_daw_core::MusicalTime;

use super::TrackBaseColor;

pub struct TimelineGridState {
    /// 1.0 means the "default zoom level".
    ///
    /// The default zoom level is arbitray. Just pick whatever looks good
    /// for now.
    ///
    /// The UI may mutate this directly without an event.
    pub horizontal_zoom_level: f64,

    /// 1.0 means the "default zoom level".
    ///
    /// The default zoom level is arbitray. Just pick whatever looks good
    /// for now.
    ///
    /// The UI may mutate this directly without an event.
    pub vertical_zoom_level: f64,

    /// The position of the left side of the timeline window.
    ///
    /// The UI may mutate this directly without an event.
    ///
    /// The UI may mutate this directly without an event.
    pub left_start: MusicalTime,

    /// This is in units of "lanes", where 1.0 means the "global default lane height".
    ///
    /// This default lane height is arbitrary, just pick whatever looks good for now.
    ///
    /// The UI may mutate this directly without an event.
    pub top_start: f64,

    /// The height of all lanes that have not specified a specific height, where 1.0
    /// means the "global default lane height".
    ///
    /// The UI may mutate this directly without an event.
    pub lane_height: f64,

    /// The list of all current lanes. (Maybe start with like 100 for a new project?)
    pub lanes: Vec<LaneState>,

    /// The time of the end of the latest clip on the timeline. This can be used to
    /// properly set the horizontal scroll bar.
    pub project_length: MusicalTime,

    /// The index of the highest-indexed lane that currently has a clip on it. This
    /// can be used to properly set the vertical scroll bar.
    pub used_lanes: u32,
    // TODO: Time signature
}

pub struct LaneState {
    /// The name of this lane.
    ///
    /// This will be `None` if this just uses the default name.
    pub name: Option<String>,

    /// The color of this lane.
    ///
    /// This will be `None` if this just uses the default color.
    pub color: Option<TrackBaseColor>,

    /// The height of this lane (where 1.0 means the "global default lane height").
    ///
    /// If this is `None`, then this will use `TimelineGridState::lane_height`
    /// instead.
    ///
    /// The UI may mutate this directly without an event.
    pub height: Option<f64>,

    /// If this is false, then it means that all clips on this lane are bypassed,
    /// so gray-out this lane.
    pub active: bool,
}
