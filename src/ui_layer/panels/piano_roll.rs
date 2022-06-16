use vizia::prelude::*;
use vizia::vg::{Paint, Path};

pub struct MidiNote {}

pub struct MidiClip {}

pub fn piano_roll(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            Label::new(cx, "PIANO ROLL").class("small");
        })
        .class("header");

        // Contents
        VStack::new(cx, |_| {}).class("level3");
    })
    .row_between(Pixels(1.0))
    .class("piano_roll");
}
