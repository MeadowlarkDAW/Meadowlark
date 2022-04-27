use vizia::vg::{Color, Paint, Path};
use vizia::*;

/// The direction the meter bar shows the peak in.
///
/// This is also used to decide the orientation of the meter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum Direction {
    /// The standard vertical meter direction
    North,
    /// The inverted direction from the standard vertical meter
    South,
    /// The standard horizontal meter direction
    East,
    /// The inverted direction from the standard horizontal meter
    West,
    /// Calculate the direction.
    /// By default the horizontal meter will move east and the vertical meter will move north
    Calculated
}

enum InternalDirection {
    /// The standard vertical meter direction
    North,
    /// The inverted direction from the standard vertical meter
    East,
    /// The standard horizontal meter direction
    South,
    /// The inverted direction from the standard horizontal meter
    West,
}

/// The different events that can be called to update states in the meter
#[derive(Debug, Clone)]
pub enum MeterEvents {
    /// Update the input value
    /// This also automatically smooths out the input and sets the max peak
    UpdatePosition(f32),
    /// Change the scale that is used to map the meter positions
    ChangeMeterScale(MeterScale),
    /// Change the amount of smoothing that is applied to the meter.
    /// Lower - More smoothing
    /// Higher - Less smoothing
    ChangeSmoothingFactor(f32),
    /// Change the speed at which the max peak drops
    ChangePeakDropSpeed(f32),
    /// Change the duration that the max peak stays in place
    ChangeMaxHoldTime(i32),
    /// Change the colour of the main peak bar
    ChangeBarColor(vizia::Color),
    /// Change the colour of the max peak line
    ChangeLineColor(vizia::Color),
    /// Change the coloured sections
    ChangeSections(Vec<(f32, f32, vizia::Color)>),
    /// Change the direction of the meter
    ChangeDirection(Direction)
}

/// Different scales to map the values with
#[derive(Debug, Clone, Copy)]
pub enum MeterScale {
    /// A linear one to one representation
    Linear,
    /// A logarithmic approximation
    /// f(x) = x^0.25
    Logarithmic,
}

/// A meter represents input values in a range of \[0,1\].
/// As an input it requires a lens. By default it scales the input values logarithmically.
/// This can be changed using a handle.
///
/// It allows you to show them as a bar that grows in each cardinal direction.
///
/// By default it smooths out the input values. The amount of smoothing can be controlled using the `smoothing_factor(f32)` handle.
/// The value should be in (0,1\] where a value of 1.0 disables smoothing. The lower the value, the stronger the smoothing.
///
/// Example:
/// ```rust
/// Data{input: 0.42}.build(cx);
///
/// // Simple meter
/// Meter::new(cx, Data::input, Direction::LeftToRight);
///
/// // Linear Meter without smoothing
/// Meter::new(cx, Data::input, Direction::LeftToRight)
/// .scale(MeterScale::Linear)
/// .smoothing_factor(1.0);
/// ```
#[derive(Lens)]
pub struct Meter {
    /// The position of the meter bar in [0,1]
    pos: f32,
    /// The scale that is used to map the input to the meter
    scale: MeterScale,
    /// The position of the max peak in [0,1]
    max: f32,
    /// A ticker to keep track of when the max peak should start dropping
    max_delay_ticker: i32,
    /// The speed at which the max peak drops
    max_drop_speed: f32,
    /// The time that the max peak should stand still for
    max_hold_time: i32,
    /// The smoothing factor in (0,1]
    smoothing_factor: f32,
    /// The direction the peak meter should grow in
    direction: Direction,
    /// The colour of the meter bar
    //NOTE: Replace this by custom style properties once they're implemented
    bar_color: vizia::Color,
    /// The colour of the peak line
    //NOTE: Replace this by custom style properties once they're implemented
    line_color: vizia::Color,
    /// The sections denoting where the bar changes colours
    /// (start, stop, colour)
    sections: Vec<(f32, f32, vizia::Color)>,
}

impl Meter {
    pub fn new<L: Lens<Target = f32>>(
        cx: &mut Context,
        lens: L,
    ) -> Handle<Self> {
        // Default values for the sections. The positions are pretty arbitrary
        let mut sections = Vec::new();
        sections.push((0.0, 0.4, vizia::Color::rgb(0, 244, 70)));
        sections.push((0.4, 0.6, vizia::Color::rgb(244, 220, 0)));
        sections.push((0.6, 0.8, vizia::Color::rgb(244, 132, 0)));
        sections.push((0.8, 1.0, vizia::Color::rgb(245, 78, 71)));

        Self {
            pos: lens.get(cx),
            scale: MeterScale::Logarithmic,
            max: 0.0,
            max_delay_ticker: 0,
            max_drop_speed: 0.006,
            max_hold_time: 25,
            smoothing_factor: 0.05,
            direction: Direction::Calculated,
            bar_color: vizia::Color::red(),
            line_color: vizia::Color::black(),
            sections,
        }
        .build(cx, move |cx| {
            // Bind the input lens to the meter event to update the position
            Binding::new(cx, lens, |cx, value| {
                cx.emit(MeterEvents::UpdatePosition(value.get(cx)));
            });
        })
    }
}

impl View for Meter {
    fn element(&self) -> Option<String> {
        Some("meter".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|meter_event, _| {
            match meter_event {
                MeterEvents::UpdatePosition(n) => {
                    let new_pos = match self.scale {
                        MeterScale::Linear => (*n).abs(),
                        MeterScale::Logarithmic => {
                            // Logarithmic approximation for 60db dynamic range
                            // Source: https://www.dr-lex.be/info-stuff/volumecontrols.html
                            (*n).abs().powf(0.25)
                        }
                    };

                    // Smoothing source: https://stackoverflow.com/a/39417788
                    // Essentially it closes in to the new position by
                    // subtracting the difference between the current position and new position
                    // and multiplying that by the smoothing_factor.
                    // This a smaller factor causes stronger smoothing.
                    // NOTE: Maybe use (1.0 - smoothing_factor) at some point to allow the factor to create the least amount of smoothing at 0.0
                    self.pos = self.pos - self.smoothing_factor * (self.pos - new_pos);

                    // If the new position is higher than the current max peak update it
                    if self.max < self.pos {
                        self.max = self.pos;
                        self.max_delay_ticker = self.max_hold_time;
                    }

                    // Once the ticker for the max peak is done start dropping it until it reaches 0
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
                MeterEvents::ChangeSections(sec) => {
                    self.sections = (*sec).to_owned();
                }
                MeterEvents::ChangeDirection(dir) => {
                    self.direction = *dir;
                }
            }
        });
    }

    fn draw(&self, cx: &mut DrawContext<'_>, canvas: &mut Canvas) {
        let entity = cx.current();

        let bounds = cx.cache().get_bounds(entity);

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

        let pos_x = cx.cache().get_posx(entity);
        let pos_y = cx.cache().get_posy(entity);
        let value = self.pos;
        let max = self.max;

        let opacity = cx.cache().get_opacity(entity);

        let mut bar_color: Color = self.bar_color.into();
        bar_color.set_alphaf(bar_color.a * opacity);

        let mut line_color: Color = self.line_color.into();
        line_color.set_alphaf(line_color.a * opacity);

        // Calculate the border radiuses
        // This is taken from the default draw implementation of Views
        let border_radius_top_left = cx
            .border_radius_top_left(entity)
            .unwrap_or_default()
            .value_or(bounds.w.min(bounds.h), 0.0);

        let border_radius_top_right = cx
            .border_radius_top_right(entity)
            .unwrap_or_default()
            .value_or(bounds.w.min(bounds.h), 0.0);

        let border_radius_bottom_left = cx
            .border_radius_bottom_left(entity)
            .unwrap_or_default()
            .value_or(bounds.w.min(bounds.h), 0.0);
        let border_radius_bottom_right = cx
            .border_radius_bottom_right(entity)
            .unwrap_or_default()
            .value_or(bounds.w.min(bounds.h), 0.0);

        // Convert the user-side direction into the internal direction without the calculated state
        let direction = match self.direction {
            Direction::North => InternalDirection::North,
            Direction::South => InternalDirection::South,
            Direction::East => InternalDirection::East,
            Direction::West => InternalDirection::West,
            Direction::Calculated => {
                if width > height {
                    InternalDirection::East
                } else {
                    InternalDirection::North
                }
            },
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

        let grad_x_start;
        let grad_x_end;
        let grad_y_start;
        let grad_y_end;

        // Build the start and end positions of the back and bar line
        // according to the direction the meter is going and the value the meter is showing
        match direction {
            InternalDirection::North => {
                bar_x = pos_x;
                bar_y = pos_y + (1.0 - value) * height;

                bar_w = width;
                bar_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + (1.0 - max) * height;
                line_y2 = line_y1;

                grad_x_start = pos_x;
                grad_x_end = pos_x;

                grad_y_start = pos_y + height;
                grad_y_end = pos_y;
            }
            InternalDirection::South => {
                bar_x = pos_x;
                bar_y = pos_y;

                bar_w = width;
                bar_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + max * height;
                line_y2 = line_y1;

                grad_x_start = pos_x;
                grad_x_end = pos_x;

                grad_y_start = pos_y;
                grad_y_end = pos_y + height;
            }
            InternalDirection::East => {
                bar_x = pos_x;
                bar_y = pos_y;

                bar_w = value * width;
                bar_h = height;

                line_x1 = pos_x + max * width;
                line_x2 = pos_x + max * width;

                line_y1 = pos_y;
                line_y2 = pos_y + height;

                grad_x_start = pos_x;
                grad_x_end = pos_x + width;

                grad_y_start = pos_y;
                grad_y_end = pos_y;
            }
            InternalDirection::West => {
                bar_x = pos_x + (1.0 - value) * width;
                bar_y = pos_y;

                bar_w = value * width;
                bar_h = height;

                line_x1 = pos_x + (1.0 - max) * width;
                line_x2 = pos_x + (1.0 - max) * width;

                line_y1 = pos_y;
                line_y2 = pos_y + height;

                grad_x_start = pos_x + width;
                grad_x_end = pos_x;

                grad_y_start = pos_y;
                grad_y_end = pos_y;
            }
        };

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

        // Convert our sections into a list femtovg can use
        let mut femtovg_sections: Vec<(f32, vizia::vg::Color)> = Vec::new();

        for (start, stop, col) in &self.sections {
            femtovg_sections.push((*start, (*col).into()));
            femtovg_sections.push((*stop, (*col).into()));
        }

        // Draw the gradient
        let mut bar_paint = Paint::linear_gradient_stops(
            grad_x_start,
            grad_y_start,
            grad_x_end,
            grad_y_end,
            &femtovg_sections,
        );

        canvas.fill_path(&mut bar_path, bar_paint);

        // Draw the peak line
        let mut line_path = Path::new();
        line_path.move_to(line_x1, line_y1);
        line_path.line_to(line_x2, line_y2);

        let mut line_paint = Paint::color(line_color);
        line_paint.set_line_width(2.0);

        canvas.stroke_path(&mut line_path, line_paint)
    }
}

pub trait MeterHandle {
    fn peak_drop_speed(self, val: impl Res<f32>) -> Self;
    fn smoothing_factor(self, val: impl Res<f32>) -> Self;
    fn bar_color(self, val: impl Res<vizia::Color>) -> Self;
    fn max_hold_time(self, val: impl Res<i32>) -> Self;
    fn line_color(self, val: impl Res<vizia::Color>) -> Self;
    fn scale(self, val: impl Res<MeterScale>) -> Self;
    fn sections(self, val: impl Res<Vec<(f32, f32, vizia::Color)>>) -> Self;
    fn direction(self, val: impl Res<Direction>) -> Self;
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

    fn sections(self, val: impl Res<Vec<(f32, f32, vizia::Color)>>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, mut value| {
            entity.emit(cx, MeterEvents::ChangeSections(value));
        });

        self
    }

    fn direction(self, val: impl Res<Direction>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, mut value| {
            entity.emit(cx, MeterEvents::ChangeDirection(value));
        });

        self
    }
}