use vizia::prelude::*;

use crate::ui::icon::{Icon, IconCode};

pub fn top_bar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        MenuController::new(cx, false, |cx| {
            MenuStack::new_horizontal(cx, |cx| {
                Menu::new(
                    cx,
                    |cx| Label::new(cx, "File"),
                    |cx| {
                        MenuButton::new_simple(cx, "Open Project", |_| {});
                        MenuButton::new_simple(cx, "Save", |_| {});
                        MenuButton::new_simple(cx, "Save As", |_| {});
                    },
                );

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "Edit").class("menu_bar_label"),
                    |cx| {
                        MenuButton::new_simple(cx, "TODO", |_| {});
                    },
                );

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "View"),
                    |cx| {
                        MenuButton::new_simple(cx, "TODO", |_| {});
                    },
                );

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "Help"),
                    |cx| {
                        MenuButton::new_simple(cx, "About", |_| {});
                    },
                );
            });
        });

        Element::new(cx).left(Pixels(10.0)).right(Pixels(10.0)).class("separator");

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Undo, 28.0, 18.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Redo, 28.0, 18.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Save, 28.0, 18.0))
                .class("icon_btn");
        })
        .class("top_bar_group_1");

        Element::new(cx).left(Pixels(10.0)).right(Pixels(10.0)).class("separator");

        HStack::new(cx, |cx| {
            Label::new(cx, "120.0 bpm");
            Label::new(cx, "TAP");
        })
        .class("top_bar_tempo");

        Element::new(cx).left(Pixels(10.0)).right(Pixels(10.0)).class("separator");

        HStack::new(cx, |cx| {
            Dropdown::new(
                cx,
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "GRV").right(Pixels(5.0));
                        Icon::new(cx, IconCode::Dropdown, 14.0, 12.0).child_top(Pixels(1.0));
                    })
                },
                |cx| {},
            )
            .width(Pixels(50.0));

            Label::new(cx, "4 / 4").top(Stretch(1.0)).bottom(Stretch(1.0)).font_size(14.0);
        })
        .class("top_bar_groove");

        Element::new(cx).left(Pixels(10.0)).right(Pixels(10.0)).class("separator");

        HStack::new(cx, |cx| {
            Label::new(cx, "2.1.1").top(Stretch(1.0)).bottom(Stretch(1.0)).class("small_text");

            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Loop, 28.0, 18.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Stop, 28.0, 18.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Play, 28.0, 18.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Record, 28.0, 18.0))
                .class("record_btn");
        })
        .left(Stretch(1.0))
        .right(Stretch(1.0))
        .position_type(PositionType::SelfDirected)
        .class("top_bar_transport");
    })
    .class("top_bar");
}
