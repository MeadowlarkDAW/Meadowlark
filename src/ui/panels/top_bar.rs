use vizia::prelude::*;

use crate::ui::icon::{Icon, IconCode};

pub fn top_bar(cx: &mut Context) {
    const SEPARATOR_PADDING: f32 = 8.0;

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

        Element::new(cx)
            .left(Pixels(SEPARATOR_PADDING))
            .right(Pixels(SEPARATOR_PADDING))
            .class("separator");

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Save, 28.0, 22.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Undo, 28.0, 22.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Redo, 28.0, 22.0))
                .class("icon_btn");
        })
        .col_between(Pixels(3.0))
        .width(Auto);

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Loop, 28.0, 22.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Stop, 28.0, 22.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Play, 28.0, 22.0))
                .class("icon_btn");
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Record, 28.0, 22.0))
                .class("record_btn");
        })
        .left(Stretch(1.0))
        .right(Stretch(1.0))
        .position_type(PositionType::SelfDirected)
        .col_between(Pixels(3.0))
        .width(Auto);

        HStack::new(cx, |cx| {
            Label::new(cx, "120.0 bpm");
            Label::new(cx, "TAP");
        })
        .left(Stretch(1.0))
        .col_between(Pixels(8.0))
        .width(Auto);

        Element::new(cx)
            .left(Pixels(SEPARATOR_PADDING))
            .right(Pixels(SEPARATOR_PADDING))
            .class("separator");

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "GRV")
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .right(Pixels(5.0));
                        Icon::new(cx, IconCode::MenuSmall, 20.0, 18.0);
                    })
                    .child_space(Pixels(5.0))
                },
            )
            .class("icon_btn")
            .width(Pixels(50.0));

            Label::new(cx, "4 / 4").font_size(14.0).top(Stretch(1.0)).bottom(Stretch(1.0));
        })
        .width(Auto)
        .col_between(Pixels(8.0))
        .right(Pixels(SEPARATOR_PADDING));
    })
    .class("top_bar");
}
