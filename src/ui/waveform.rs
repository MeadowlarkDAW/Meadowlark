use std::path::PathBuf;

use symphonia::{core::sample::SampleFormat, default::formats::WavReader};
use vizia::*;

pub struct Waveform {}

impl Waveform {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {}.build2(cx, |cx| {})
    }
}

impl View for Waveform {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}

    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {}
}
