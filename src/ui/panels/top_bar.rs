use vizia::*;
use crate::ui::meter::{Meter, Direction};

#[derive(Lens)]
pub struct Data {
    input_l: f32,
    input_r: f32
}

impl Model for Data {}

use crate::ui::{icons::IconCode, Icon, PanelEvent};

pub fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Button::new(
            cx,
            |cx| {},
            |cx| Icon::new(cx, IconCode::Menu, 24.0, 16.0),
        )
        .class("top_bar_menu");

        // This is all just dummy content and it doesn't do anything
        HStack::new(cx, |cx |{

            VStack::new(cx, |cx|{
                HStack::new(cx, |cx| {
                    Label::new(cx, "130.00");
                    Label::new(cx, "TAP");
                });
                HStack::new(cx, |cx| {
                    Label::new(cx, "4/4");
                    Label::new(cx, "GRV");
                });
            })
            .class("top_play_left");

            HStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |cx| {},
                    |cx| Icon::new(cx, IconCode::Play, 24.0, 23.0),
                );
                Button::new(
                    cx,
                    |cx| {},
                    |cx| Icon::new(cx, IconCode::Stop, 24.0, 23.0),
                );
                Button::new(
                    cx,
                    |cx| {},
                    |cx| Icon::new(cx, IconCode::Record, 24.0, 23.0),
                );
            })
            .class("top_play_center")
            .top(Stretch(1.0))
            .bottom(Stretch(1.0));

            VStack::new(cx, |cx|{
                Label::new(cx, "AUDIO");
                Label::new(cx, "OVERWRITE");
            })
            .class("top_play_right");
            
        })
        .class("top_bar_play");

        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Button::new(
                        cx,
                        |cx| {},
                        |cx| Icon::new(cx, IconCode::Hierarchy, 24.0, 16.0),
                    );
                    Button::new(
                        cx,
                        |cx| {},
                        |cx| Icon::new(cx, IconCode::Grid, 24.0, 16.0),
                    );
                    Button::new(
                        cx,
                        |cx| {},
                        |cx| Icon::new(cx, IconCode::Mixer, 24.0, 16.0),
                    );
                    Button::new(
                        cx,
                        |cx| cx.emit(PanelEvent::TogglePianoRoll),
                        |cx| Icon::new(cx, IconCode::Piano, 24.0, 16.0),
                    );
                });
                HStack::new(cx, |cx| {
                    Button::new(
                        cx,
                        |cx| {},
                        |cx| Icon::new(cx, IconCode::Automation, 24.0, 16.0),
                    );
                    Button::new(
                        cx,
                        |cx| {},
                        |cx| Icon::new(cx, IconCode::Sample, 24.0, 16.0),
                    );
                    Button::new(
                        cx,
                        |cx| {},
                        |cx| Icon::new(cx, IconCode::DrumSequencer, 24.0, 16.0),
                    );
                    Button::new(
                        cx,
                        |cx| {},
                        |cx| Icon::new(cx, IconCode::Stack, 24.0, 16.0),
                    );
                });
            })
            .class("top_bar_select_container");
            
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "Oscilloscope");
                    VStack::new(cx, |cx| {

                        Data{input_l: 0.42, input_r: 0.69}.build(cx);
                        Meter::new(cx, Data::input_l, Direction::LeftToRight)
                        .class("top_bar_peak");
                        Meter::new(cx, Data::input_r, Direction::LeftToRight)
                        .class("top_bar_peak");

                    })
                    .class("top_bar_peak_container");

                })
                .class("top_bar_audio_graph_container");
                
                VStack::new(cx, |cx| {
                    Label::new(cx, "Usage Graph")
                    .top(Stretch(1.0))
                    .bottom(Stretch(1.0));
                })
                .class("top_bar_usage_graph_container");
                
            })
            .class("top_bar_graph_container");
        })
        .class("top_bar_right_container");
    })
    .class("top_bar");
}
