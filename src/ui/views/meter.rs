use vizia::{vg::femtovg::{Path, Paint, Color}};
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

#[derive(Debug)]
pub enum MeterEvents {
    UpdatePosition(f32),
    ChangePeakDropSpeed(f32),
    ChangeSmoothingFactor(f32),
    ChangeMaxHoldTime(i32),
    ChangeBarColor(String)
}

#[derive(Lens)]
pub struct Meter {
    pos: f32,
    max: f32,
    max_delay_ticker: i32,
    max_drop_speed: f32,
    max_hold_time: i32,
    smoothing_factor: f32,
    direction: Direction,
    bar_color: String,
    line_color: String
}

impl Meter {
    pub fn new<L: Lens<Target = f32>>(cx: &mut Context, lens: L, direction: Direction) -> Handle<Self> {
        vizia::View::build(Self {
            pos: 0.0,
            max: 0.0,
            max_delay_ticker: 0,
            max_drop_speed: 0.05,
            max_hold_time: 50,
            smoothing_factor: 0.1,
            direction,
            bar_color: String::from("#ff0000"),
            line_color: String::from("#000000")
        }, cx, move |cx| {
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

    fn event(&mut self, _cx: &mut Context, event: &mut Event) {
        if let Some(param_change_event) = event.message.downcast() {
            match param_change_event {
                MeterEvents::UpdatePosition(n) => {
                    self.pos = self.pos - self.smoothing_factor * (self.pos - (*n).abs());

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
                    self.bar_color = col.clone();
                }
                
            }
        }
    }

    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let width = cx.cache.get_width(cx.current);
        let height = cx.cache.get_height(cx.current);
        let pos_x = cx.cache.get_posx(cx.current);
        let pos_y = cx.cache.get_posy(cx.current);
        let value = self.pos;
        let max = self.max;

        // Create variables for the rectangle
        let front_x;
        let front_y;
        let front_w;
        let front_h;

        let line_x1;
        let line_x2;
        let line_y1;
        let line_y2;

        // Build the start and end positions of the back and front line
        // according to the direction the meter is going and the value the meter is showing
        match self.direction {
            Direction::DownToUp => {
                front_x = pos_x;
                front_y = pos_y + (1.0-value) * height;

                front_w = width;
                front_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + (1.0-max) * height;
                line_y2 = line_y1;
            },
            Direction::UpToDown => {
                front_x = pos_x;
                front_y = pos_y;

                front_w = width;
                front_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + max * height;
                line_y2 = line_y1;
            },
            Direction::LeftToRight => {
                front_x = pos_x;
                front_y = pos_y;

                front_w = value * width;
                front_h = height;

                line_x1 = pos_x + max * width;
                line_x2 = pos_x + max * width;

                line_y1 = pos_y;
                line_y2 = pos_y + height;
            },
            Direction::RightToLeft => {
                front_x = pos_x + (1.0-value) * width;
                front_y = pos_y;

                front_w = value * width;
                front_h = height;

                line_x1 = pos_x + (1.0 - max) * width;
                line_x2 = pos_x + (1.0 - max) * width;

                line_y1 = pos_y;
                line_y2 = pos_y + height;
            },
            _ => {
                front_x = pos_x;
                front_y = pos_y + (1.0-value) * height;

                front_w = width;
                front_h = value * height;

                line_x1 = pos_x;
                line_x2 = pos_x + width;

                line_y1 = pos_y + max * height;
                line_y2 = line_y1;
            }
        };


        // Draw the bar
        if value >= 1e-3 {
            let mut front_path = Path::new();
            front_path.rect(front_x, front_y, front_w, front_h);

            let mut front_paint = Paint::color(Color::hex(self.bar_color.as_str()));

            canvas.fill_path(&mut front_path, front_paint);

            let mut line_path = Path::new();
            line_path.move_to(line_x1, line_y1);
            line_path.line_to(line_x2, line_y2);

            let mut line_paint = Paint::color(Color::hex(self.line_color.as_str()));
            line_paint.set_line_width(2.0);

            canvas.stroke_path(&mut line_path, line_paint)
        }
    }
}

pub trait MeterHandle {
    fn peak_drop_speed(self, val: impl Res<f32>) -> Self;
    fn smoothing_factor(self, val: impl Res<f32>) -> Self;
    fn bar_color(self, val: impl Res<String>) -> Self;
    fn max_hold_time(self, val: impl Res<i32>) -> Self;
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

    fn bar_color(self, val: impl Res<String>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangeBarColor(value));
        });

        self
    }

    fn max_hold_time(self, val: impl Res<i32>) -> Self {
        val.set_or_bind(self.cx, self.entity, |cx, entity, value| {
            entity.emit(cx, MeterEvents::ChangeMaxHoldTime(value));
        });

        self
    }
}