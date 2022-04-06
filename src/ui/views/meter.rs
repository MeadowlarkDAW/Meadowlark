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
    Radial,
}

#[derive(Lens)]
pub struct MeterData {
    pos: f32,
    max: f32,
    max_delay_ticker: i32,
    max_drop_speed: f32,
    smoothing_factor: f32,
}

impl Model for MeterData {
    fn event(&mut self, _cx: &mut Context, event: &mut Event) {
        if let Some(param_change_event) = event.message.downcast() {
            match param_change_event {
                MeterEvents::UpdatePosition(n) => {
                    self.pos = self.pos - self.smoothing_factor * (self.pos - (*n).abs());

                    if self.max < self.pos {
                        self.max = self.pos;
                        self.max_delay_ticker = 50;
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
            }
        }
    }
}

#[derive(Debug)]
pub enum MeterEvents {
    UpdatePosition(f32),
    ChangePeakDropSpeed(f32),
    ChangeSmoothingFactor(f32),
}

pub struct Meter<L> {
    /// The position of the meter in [0,1]
    lens: L
}

impl<L: Lens<Target = f32>> Meter<L> {
    pub fn new(cx: &mut Context, lens: L, direction: Direction) -> Handle<Self> {
        Self {
            lens: lens.clone()
        }
        .build2(cx, move |cx| {
            MeterData {
                pos: 0.0,
                max: 0.0,
                max_delay_ticker: 0,
                max_drop_speed: 0.05,
                smoothing_factor: 0.1,
            }
            .build(cx);

            Binding::new(cx, lens, |cx, value| {
                cx.emit(MeterEvents::UpdatePosition(value.get(cx)));
            });
            ZStack::new(cx, |cx| {

                // Draws the peak meter in the different directions. This currently is set at the initialization and can't be changed
                match direction {
                    Direction::DownToUp => {
                        Element::new(cx)
                            .height(MeterData::pos.map(|val| Percentage(val * 100.0)))
                            .top(Stretch(1.0))
                            .width(Stretch(1.0))
                            .class("meter_bar");

                        Element::new(cx)
                            .width(Stretch(1.0))
                            .height(Pixels(2.0))
                            .top(Stretch(1.0))
                            .bottom(MeterData::max.map(|val| Percentage(val * 100.0)))
                            .class("meter_line");
                    }
                    Direction::UpToDown => {
                        Element::new(cx)
                            .height(MeterData::pos.map(|val| Percentage(val * 100.0)))
                            .bottom(Stretch(1.0))
                            .width(Stretch(1.0))
                            .class("meter_bar");

                        Element::new(cx)
                            .width(Stretch(1.0))
                            .height(Pixels(2.0))
                            .bottom(Stretch(1.0))
                            .top(MeterData::max.map(|val| Percentage(val * 100.0)))
                            .class("meter_line");
                    }
                    Direction::LeftToRight => {
                        Element::new(cx)
                            .width(MeterData::pos.map(|val| Percentage(val * 100.0)))
                            .right(Stretch(1.0))
                            .height(Stretch(1.0))
                            .class("meter_bar");

                        Element::new(cx)
                            .height(Stretch(1.0))
                            .width(Pixels(2.0))
                            .right(Stretch(1.0))
                            .left(MeterData::max.map(|val| Percentage(val * 100.0)))
                            .class("meter_line");
                    }
                    Direction::RightToLeft => {
                        Element::new(cx)
                            .width(MeterData::pos.map(|val| Percentage(val * 100.0)))
                            .left(Stretch(1.0))
                            .height(Stretch(1.0))
                            .class("meter_bar");

                        Element::new(cx)
                            .height(Stretch(1.0))
                            .width(Pixels(2.0))
                            .left(Stretch(1.0))
                            .right(MeterData::max.map(|val| Percentage(val * 100.0)))
                            .class("meter_line");
                    }
                    Direction::Radial => {
                        // This is a circle :)
                        Element::new(cx)
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .left(Stretch(1.0))
                            .right(Stretch(1.0))
                            .height(MeterData::pos.map(|val| Percentage(val * 100.0)))
                            .width(MeterData::pos.map(|val| Percentage(val * 100.0)))
                            .border_radius(Percentage(50.0))
                            .class("meter_bar");
                    }
                }
            });
        })
    }
}

impl<L: Lens<Target = f32>> View for Meter<L> {
    fn element(&self) -> Option<String> {
        Some("meter".to_string())
    }
}

pub trait MeterHandle {
    fn peak_drop_speed(self, val: impl Res<f32>) -> Self;
    fn smoothing_factor(self, val: impl Res<f32>) -> Self;
}

impl<T> MeterHandle for Handle<'_, Meter<T>> {
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
}
