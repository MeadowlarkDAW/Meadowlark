use vizia::*;
use crate::ui::meter::{Meter, Direction};

#[derive(Lens)]
pub struct Data {
    input_l: f32,
    input_r: f32
}

impl Model for Data {}

pub fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, "Menu");
        HStack::new(cx, |_| {}).text("Top Buttons");
        HStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Element::new(cx).text("Graphs");
                    VStack::new(cx, |cx| {
                        Data{input_l: 0.69, input_r: 0.42}.build(cx);
                        Meter::new(cx, Data::input_l, Direction::LeftToRight)
                            .class("top_bar_peak_meter");
                        Meter::new(cx, Data::input_r, Direction::LeftToRight)
                            .class("top_bar_peak_meter");
                    }).class("top_bar_peak_meter_stack");
                });
                Element::new(cx).text("Graphs");
            });
        })
        .class("Graphs");
    }).class("top_bar");
}
