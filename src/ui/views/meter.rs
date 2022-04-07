use vizia::vg::{Color, Paint, Path};
use vizia::*;

/// The direction the meter bar shows the peak in.
/// The semantic is LowToHigh, so DownToUp is the standard vertical meter design
///
/// This is also used to decide the orientation of the meter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum Direction {
    /// The standard vertical meter direction
    DownToUp,
    /// The inverted direction from the standard vertical meter
    UpToDown,
    /// The standard horizontal meter direction
    LeftToRight,
    /// The inverted direction from the standard horizontal meter
    RightToLeft,
    /// A special round peak meter
    Radial,
}

#[derive(Debug, Clone, Copy)]
pub enum MeterEvents {
    UpdatePosition(f32),
    ChangeMeterScale(MeterScale),
    ChangePeakDropSpeed(f32),
    ChangeSmoothingFactor(f32),
    ChangeMaxHoldTime(i32),
    ChangeBarColor(vizia::Color),
    ChangeLineColor(vizia::Color),
}

#[derive(Debug, Clone, Copy)]
pub enum MeterScale {
    Linear,
    Logarithmic
}

#[derive(Lens)]
pub struct Meter {
    pos: f32,
    prev_pos: f32,
    scale: MeterScale,
    max: f32,
    max_delay_ticker: i32,
    max_drop_speed: f32,
    max_hold_time: i32,
    smoothing_factor: f32,
    direction: Direction,
    bar_color: vizia::Color,
    line_color: vizia::Color,
}

impl Meter {
    pub fn new<L: Lens<Target = f32>>(
        cx: &mut Context,
        lens: L,
        direction: Direction,
    ) -> Handle<Self> {
        vizia::View::build(
            Self {
                pos: lens.get(cx),
                prev_pos: 0.0,
                scale: MeterScale::Logarithmic,
                max: 0.0,
                max_delay_ticker: 0,
                max_drop_speed: 0.006,
                max_hold_time: 25,
                smoothing_factor: 0.05,
                direction,
                bar_color: vizia::Color::red(),
                line_color: vizia::Color::black(),
            },
            cx,
            move |cx| {
                Binding::new(cx, lens, |cx, value| {
                    cx.emit(MeterEvents::UpdatePosition(value.get(cx)));
                });
            },
        )
    }
}

impl View for Meter {
    fn element(&self) -> Option<String> {
        Some("meter".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(meter_event) = event.message.downcast() {
            match meter_event {
                MeterEvents::UpdatePosition(n) => {
                    let new_pos = match self.scale {
                        MeterScale::Linear => {
                            (*n).abs()
                        }
                        MeterScale::Logarithmic => {
                            // Logarithmic approximation for 60db dynamic range
                            // Source: https://www.dr-lex.be/info-stuff/volumecontrols.html
                            (*n).abs().powf(0.25)
                        }
                    };

                    self.prev_pos = self.pos;
                    self.pos = self.pos - self.smoothing_factor * (self.pos - new_pos);

                    if self.max < self.pos {
                        self.max = self.pos;
                        self.max_delay_ticker = self.max_hold_time;
                    }

                    if self.max_delay_ticker == 0 {
                        self.max -= self.max_drop_speed;

                        if self.max < 0.0 {
                            self.max = 0.0;
                        }
                    } else {
                        self.max_delay_ticker -= 1;
                    }

                    cx.style.needs_redraw = true;
                }
                MeterEvents::ChangeMeterScale(scale) => {
                    self.scale = *scale;
                }
                MeterEvents::ChangePeakDropSpeed(n) => {
                    self.max_drop_speed = *n;
                }
                MeterEvents::ChangeSmoothingFactor(n) => {
                    self.smoothing_factor = *n;
                }
                MeterEvents::ChangeMaxHoldTime(n) => {
                    self.max_hold_time = *n;
                }
                MeterEvents::ChangeBarColor(col) => {
                    self.bar_color = *col;
                }
                MeterEvents::ChangeLineColor(col) => {
                    self.line_color = *col;
                }
            }
        }
    }

    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let entity = cx.current;

        let bounds = cx.cache.get_bounds(entity);

        //Skip meters with no width or no height
        if bounds.w == 0.0 || bounds.h == 0.0 {
            return;
        }
        let width = bounds.w;
        let height = bounds.h;

        // TODO:
        // Whole meter background
        // Border
        // Border shape
        // Padding
        // Space
        // Radial

        let pos_x = cx.cache.get_posx(entity);
        let pos_y = cx.cache.get_posy(entity);
        let value = self.pos;
        let prev_value = self.prev_pos;
        let max = self.max;

        let opacity = cx.cache.get_opacity(entity);

        let mut bar_color: Color = self.bar_color.into();
        bar_color.set_alphaf(bar_color.a * opacity);

        let mut line_color: Color = self.line_color.into();
        line_color.set_alphaf(line_color.a * opacity);

        let border_radius_top_left = match cx
            .style
            .border_radius_top_left
            .get(entity)
            .cloned()
            .unwrap_or_default()
        {
            Units::Pixels(val) => val,
            Units::Percentage(val) => bounds.w.min(bounds.h) * (val / 100.0),
            _ => 0.0,
        };

        let border_radius_top_right = match cx
            .style
            .border_radius_top_right
            .get(entity)
            .cloned()
            .unwrap_or_default()
        {
            Units::Pixels(val) => val,
            Units::Percentage(val) => bounds.w.min(bounds.h) * (val / 100.0),
            _ => 0.0,
        };

        let border_radius_bottom_left = match cx
            .style
            .border_radius_bottom_left
            .get(entity)
            .cloned()
            .unwrap_or_default()
        {
            Units::Pixels(val) => val,
            Units::Percentage(val) => bounds.w.min(bounds.h) * (val / 100.0),
            _ => 0.0,
        };

        let border_radius_bottom_right = match cx
            .style
            .border_radius_bottom_right
            .get(entity)
            .cloned()
            .unwrap_or_default()
        {
            Units::Pixels(val) => val,
            Units::Percentage(val) => bounds.w.min(bounds.h) * (val / 100.0),
            _ => 0.0,
        };

        // Create variables for the rectangle
        let bar_x;
        let bar_y;
        let bar_w;
        let bar_h;

        let line_x1;
        let line_x2;
        let line_y1;
        let line_y2;

        // Build the start and end positions of the back and bar line
        // according to the direction the meter is going and the value the meter is showing
        match self.direction {
            Direction::DownToUp => {
                bar_x = pos_x;
                bar_y = pos_y + (1.0 - value) * height;

                bar_w = width;
                bar_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + (1.0 - max) * height;
                line_y2 = line_y1;
            }
            Direction::UpToDown => {
                bar_x = pos_x;
                bar_y = pos_y;

                bar_w = width;
                bar_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + max * height;
                line_y2 = line_y1;
            }
            Direction::LeftToRight => {
                bar_x = pos_x;
                bar_y = pos_y;

                bar_w = value * width;
                bar_h = height;

                line_x1 = pos_x + max * width;
                line_x2 = pos_x + max * width;

                line_y1 = pos_y;
                line_y2 = pos_y + height;
            }
            Direction::RightToLeft => {
                bar_x = pos_x + (1.0 - value) * width;
                bar_y = pos_y;

                bar_w = value * width;
                bar_h = height;

                line_x1 = pos_x + (1.0 - max) * width;
                line_x2 = pos_x + (1.0 - max) * width;

                line_y1 = pos_y;
                line_y2 = pos_y + height;
            }
            _ => {
                bar_x = pos_x;
                bar_y = pos_y + (1.0 - value) * height;

                bar_w = width;
                bar_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + max * height;
                line_y2 = line_y1;
            }
        };

        // Draw the meter
        if prev_value >= 1e-3 {
            let mut bar_path = Path::new();
            bar_path.rounded_rect_varying(
                bar_x,
                bar_y,
                bar_w,
                bar_h,
                border_radius_top_left,
                border_radius_top_right,
                border_radius_bottom_left,
                border_radius_bottom_right,
            );

            let mut bar_paint = Paint::color(bar_color);

            canvas.fill_path(&mut bar_path, bar_paint);

            let mut line_path = Path::new();
            line_path.move_to(line_x1, line_y1);
            line_path.line_to(line_x2, line_y2);

            let mut line_paint = Paint::color(line_color);
            line_paint.set_line_width(2.0);

            canvas.stroke_path(&mut line_path, line_paint)
        }
    }
}

pub trait MeterHandle {
    fn peak_drop_speed(self, val: impl Res<f32>) -> Self;
    fn smoothing_factor(self, val: impl Res<f32>) -> Self;
    fn bar_color(self, val: impl Res<vizia::Color>) -> Self;
    fn max_hold_time(self, val: impl Res<i32>) -> Self;
    fn line_color(self, val: impl Res<vizia::Color>) -> Self;
    fn scale(self, val: impl Res<MeterScale>) -> Self;
}

impl MeterHandle for Handle<'_, Meter> {
    fn peak_drop_speed(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangePeakDropSpeed(value));
        });

        self
    }

    fn smoothing_factor(self, val: impl Res<f32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangeSmoothingFactor(value));
        });

        self
    }

    fn bar_color(self, val: impl Res<vizia::Color>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangeBarColor(value));
        });

        self
    }

    fn line_color(self, val: impl Res<vizia::Color>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangeLineColor(value));
        });

        self
    }

    fn max_hold_time(self, val: impl Res<i32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangeMaxHoldTime(value));
        });

        self
    }

    fn scale(self, val: impl Res<MeterScale>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangeMeterScale(value));
        });

        self
    }
}

