use super::core_types::WMusicalTime;
use super::{LaneStates, UiEvent};
use vizia::prelude::*;

#[derive(Debug, Lens, Clone)]
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
    pub left_start: WMusicalTime,

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
    pub lane_states: LaneStates,

    /// The time of the end of the latest clip on the timeline. This can be used to
    /// properly set the horizontal scroll bar.
    pub project_length: WMusicalTime,

    /// The index of the highest-indexed lane that currently has a clip on it. This
    /// can be used to properly set the vertical scroll bar.
    pub used_lanes: u32,
    // TODO: Time signature
}

pub const VERTICAL_ZOOM_STEP: f64 = 0.25;
// TODO: Horizontal zoom
// pub const HORIZONTAL_ZOOM_STEP: f64 = 0.25;
pub const MINIMUM_VERTICAL_ZOOM: f64 = 0.25;
pub const MAXIMUM_VERTICAL_ZOOM: f64 = 4.0;
pub const MINIMUM_LANE_HEIGHT: f64 = 0.25;
pub const MAXIMUM_LANE_HEIGHT: f64 = 4.0;
pub const LANE_HEIGHT_STEP: f64 = 0.25;

impl Model for TimelineGridState {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|event, _| match event {
            UiEvent::ZoomInVertically => {
                self.vertical_zoom_level =
                    (self.vertical_zoom_level + VERTICAL_ZOOM_STEP).min(MAXIMUM_VERTICAL_ZOOM);
                cx.need_redraw();
            }
            UiEvent::ZoomOutVertically => {
                self.vertical_zoom_level =
                    (self.vertical_zoom_level - VERTICAL_ZOOM_STEP).max(MINIMUM_VERTICAL_ZOOM);
                cx.need_redraw();
            }
            UiEvent::DecreaseSelectedLaneHeight => {
                for lane in self.lane_states.selected_lanes_mut() {
                    if let Some(height) = lane.height {
                        lane.height = Some((height - LANE_HEIGHT_STEP).max(MINIMUM_LANE_HEIGHT));
                    } else {
                        lane.height = Some(self.lane_height - LANE_HEIGHT_STEP);
                    }
                }
            }
            UiEvent::IncreaseSelectedLaneHeight => {
                for lane in self.lane_states.selected_lanes_mut() {
                    if let Some(height) = lane.height {
                        lane.height = Some((height + LANE_HEIGHT_STEP).min(MAXIMUM_LANE_HEIGHT));
                    } else {
                        lane.height = Some(self.lane_height + LANE_HEIGHT_STEP);
                    }
                }
            }
            _ => {}
        });
        self.lane_states.event(cx, event);
    }
}
