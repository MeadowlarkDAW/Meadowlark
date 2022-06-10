use piano_keyboard::KeyboardBuilder;
use vizia::prelude::*;
use vizia::vg::{Path, Paint};

use crate::ui::PanelState;

pub struct MidiNote {

}

pub struct MidiClip {

}

pub fn piano_roll(cx: &mut Context) {
    VStack::new(cx, |cx| {
        HStack::new(cx, |_| {}).class("toolbar");
        PianoWidget::new(cx);
    })
    .class("piano_roll")
    .toggle_class("hidden", PanelState::hide_piano_roll);
}


pub struct PianoWidget {
    pub key_height: f32,
}

impl PianoWidget {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            key_height: 10.0,
        }.build(cx, |_|{})
    }
}

impl View for PianoWidget {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        // FIX THIS LATER (VIZIA)
        let current = cx.current();
        let bounds = cx.cache().get_bounds(current);

        let keyboard = KeyboardBuilder::new().standard_piano(25).unwrap().set_width(bounds.h as u16).unwrap().build2d();

        let mut path = Path::new();
        // for key in keyboard.iter() {
        //     key.
        // }
        for key in keyboard.white_keys(false) {
            path.rect(bounds.x + key.y as f32, bounds.y + key.x as f32, key.height as f32, key.width as f32);
        }

        canvas.fill_path(&mut path, Paint::color(Color::white().into()));
        //draw_octave(cx, canvas, bounds.h, 140.0, bounds.x, bounds.y);

    }
}

fn draw_octave(cx: &mut DrawContext, canvas: &mut Canvas, height: f32, width: f32, posx: f32, posy: f32) {

    let dpi = 2.0;

    let corner_radius = 2.0 * dpi;
    let spacing = 0.0 * dpi;

    let height_black_key = height / 12.0;
    let height_white_key = height / 7.0;

    let short_white = ((23.0 / 164.0) * height).round();
    let long_white = ((24.0 / 164.0) * height).round();

    let short_black = ((13.0 / 164.0) * height).round();
    let long_black = ((14.0 / 164.0) * height).round();

    let mut path = Path::new();
    let mut py = posy;
    for i in 0..7 {
        let height = match i {
            0 | 2 | 4 | 5 => {
                short_white
            }
            
            _=> {
                long_white
            }
        };
        path.rounded_rect_varying(posx, py, width, height, 0.0, corner_radius, corner_radius, 0.0);
        py += height;
    }

    // path.rounded_rect_varying(posx, posy + short_white, width, height * long_white, 0.0, corner_radius, corner_radius, 0.0);
    // path.rounded_rect_varying(posx, posy + long_white, width, height * short_white, 0.0, corner_radius, corner_radius, 0.0);
    // path.rounded_rect_varying(posx, posy + short_white, width, height * long_white, 0.0, corner_radius, corner_radius, 0.0);
    // path.rounded_rect_varying(posx, posy + long_white, width, height * short_white, 0.0, corner_radius, corner_radius, 0.0);
    // path.rounded_rect_varying(posx, posy + short_white, width, height * short_white, 0.0, corner_radius, corner_radius, 0.0);
    // path.rounded_rect_varying(posx, posy + short_white, width, height * long_white, 0.0, corner_radius, corner_radius, 0.0);

    canvas.fill_path(&mut path, Paint::color(Color::white().into()));

    
    // for i in 0..7 {
    //     let mut path = Path::new();
    //     path.rounded_rect_varying(posx, posy + height_white_key * i as f32, width, height_white_key - spacing, 0.0, corner_radius, corner_radius, 0.0);
    //     canvas.fill_path(&mut path, Paint::color(Color::rgb(50 * i, 0, 0).into()));
    // }

    // let mut path = Path::new();
    // for i in 0..12 {
    //     match i {
    //         1 | 3 | 5 | 8 | 10 => {
    //             path.rounded_rect_varying(posx, posy + height_black_key * i as f32, 2.0*width/3.0, height_black_key - spacing, 0.0, corner_radius, corner_radius, 0.0);
    //         } 

    //         _=> {} 
    //     }
    // }
    // canvas.fill_path(&mut path, Paint::color(Color::black().into()));

}