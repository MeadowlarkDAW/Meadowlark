use super::lanes::{LaneStates, DEFAULT_LANE_HEIGHT_PX};
use crate::ui::{AppData, AppEvent};
use meadowlark_core_types::MusicalTime;
use vizia::{
    prelude::*,
    vg::{Align, Baseline, Paint, Path},
};

pub const TIMELINE_DEFAULT_OFFSET: f32 = 10.0;
pub const TIMELINE_GAP_BETWEEN_LANES: f32 = 1.0;

#[derive(Debug, Lens, Clone, Serialize, Deserialize)]
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
    pub lane_states: LaneStates,

    /// The time of the end of the latest clip on the timeline. This can be used to
    /// properly set the horizontal scroll bar.
    pub project_length: MusicalTime,

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
        event.map(|program_event, _| match program_event {
            AppEvent::ZoomInVertically => {
                self.vertical_zoom_level =
                    (self.vertical_zoom_level + VERTICAL_ZOOM_STEP).min(MAXIMUM_VERTICAL_ZOOM);
                cx.need_redraw();
            }
            AppEvent::ZoomOutVertically => {
                self.vertical_zoom_level =
                    (self.vertical_zoom_level - VERTICAL_ZOOM_STEP).max(MINIMUM_VERTICAL_ZOOM);
                cx.need_redraw();
            }
            AppEvent::DecreaseSelectedLaneHeight => {
                for lane in self.lane_states.selected_lanes_mut() {
                    if let Some(height) = lane.height {
                        lane.height = Some((height - LANE_HEIGHT_STEP).max(MINIMUM_LANE_HEIGHT));
                    } else {
                        lane.height = Some(self.lane_height - LANE_HEIGHT_STEP);
                    }
                }
            }
            AppEvent::IncreaseSelectedLaneHeight => {
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

pub struct TimelineGrid;

impl TimelineGrid {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |_| {}).focusable(false).hoverable(false)
    }
}

impl View for TimelineGrid {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let entity = cx.current();
        let bounds = cx.cache().get_bounds(entity);
        let clip_region = cx.cache().get_clip_region(entity);

        if let Some(app_data) = cx.data::<AppData>() {
            let timeline_grid = &app_data.state.timeline_grid;
            let start = timeline_grid.left_start.as_beats_f64();
            let end = timeline_grid.left_start.as_beats_f64()
                + timeline_grid.project_length.as_beats_f64();
            // TODO: Horizontal zoom
            // let zoom_x = timeline_grid.horizontal_zoom_level;
            let zoom_y = timeline_grid.vertical_zoom_level;

            canvas.save();
            canvas.scissor(bounds.x, bounds.y, bounds.w, bounds.h);

            // Horizontal lines
            let mut lane_y = 0.0;
            for (index, lane) in timeline_grid.lane_states.lanes.iter().enumerate() {
                let lane_height = (DEFAULT_LANE_HEIGHT_PX
                    * if let Some(height) = lane.height {
                        height as f32
                    } else {
                        timeline_grid.lane_height as f32
                    }
                    + TIMELINE_GAP_BETWEEN_LANES)
                    * zoom_y as f32;

                lane_y += cx.logical_to_physical(lane_height);

                // Avoid drawing lines outside of the clip region
                if bounds.y + lane_y < clip_region.y
                    || bounds.y + lane_y > clip_region.y + clip_region.h
                {
                    continue;
                }

                let mut path = Path::new();
                path.move_to(bounds.x, bounds.y + lane_y);
                path.line_to(bounds.x + bounds.w, bounds.y + lane_y);
                canvas.stroke_path(&mut path, Paint::color(vizia::vg::Color::rgb(10, 10, 10)));
            }

            // Vertical lines
            let beat_width = 100.0;
            let mut lane_x = cx.logical_to_physical(TIMELINE_DEFAULT_OFFSET);
            for index in (start as usize)..=(end as usize) {
                let mut path = Path::new();
                path.move_to(bounds.x + lane_x, clip_region.y);
                path.line_to(bounds.x + lane_x, clip_region.y + clip_region.h);
                canvas.stroke_path(&mut path, Paint::color(vizia::vg::Color::rgb(10, 10, 10)));
                lane_x += cx.logical_to_physical(beat_width);
            }
            canvas.restore();
        }
    }
}

pub struct TimelineGridHeader;

impl TimelineGridHeader {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build(cx, |_| {}).focusable(false).hoverable(false)
    }
}

impl View for TimelineGridHeader {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let entity = cx.current();
        let bounds = cx.cache().get_bounds(entity);

        // TODO: Find out how to get access to the `FontId`.
        // let font = cx.font(entity);
        // let default_font = cx.default_font();

        // let font_id = cx
        //     .resource_manager()
        //     .fonts
        //     .get(&font)
        //     .and_then(|font| match font {
        //         FontOrId::Id(id) => Some(id),
        //         _ => None,
        //     })
        //     .unwrap_or(default_font);

        if let Some(app_data) = cx.data::<AppData>() {
            let timeline_grid = &app_data.state.timeline_grid;
            let start = timeline_grid.left_start.as_beats_f64();
            let end = timeline_grid.left_start.as_beats_f64()
                + timeline_grid.project_length.as_beats_f64();

            canvas.save();
            canvas.scissor(bounds.x, bounds.y, bounds.w, bounds.h);

            // Vertical lines
            let beat_width = 100.0;
            let mut lane_x = cx.logical_to_physical(TIMELINE_DEFAULT_OFFSET);
            for index in (start as usize)..=(end as usize) {
                // Line per bar
                let mut path = Path::new();
                path.move_to(bounds.x + lane_x, bounds.y + bounds.h);
                path.line_to(bounds.x + lane_x, bounds.y + bounds.h - cx.logical_to_physical(10.0));
                canvas.stroke_path(&mut path, Paint::color(vizia::vg::Color::rgb(82, 82, 82)));

                // Number per bar
                let mut text_paint = Paint::color(vizia::vg::Color::rgb(82, 82, 82));
                // text_paint.set_font(&[font_id.clone()]);
                text_paint.set_text_align(Align::Center);
                text_paint.set_text_baseline(Baseline::Top);
                let _ = canvas.fill_text(
                    bounds.x + lane_x,
                    bounds.y,
                    &format!("{}", index + 1),
                    text_paint,
                );

                // Line per beat
                if index != end as usize {
                    // Line per bar
                    for index in 1..4 {
                        let lane_bar_x =
                            lane_x + cx.logical_to_physical(index as f32 * beat_width / 4.0);

                        let mut path = Path::new();
                        let length = cx.logical_to_physical(if index == 2 { 8.0 } else { 5.0 });

                        path.move_to(bounds.x + lane_bar_x, bounds.y + bounds.h);
                        path.line_to(bounds.x + lane_bar_x, bounds.y + bounds.h - length);
                        canvas.stroke_path(
                            &mut path,
                            Paint::color(vizia::vg::Color::rgb(82, 82, 82)),
                        );
                    }
                }

                lane_x += cx.logical_to_physical(beat_width);
            }
            canvas.restore();
        }
    }
}
