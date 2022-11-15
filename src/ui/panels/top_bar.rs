use vizia::prelude::*;

use crate::ui::views::{Icon, IconCode};

pub fn top_bar(cx: &mut Context) {
    const TOP_BAR_HEIGHT: f32 = 36.0;
    const TOP_BAR_CHILD_SPACE: f32 = 2.0;

    const TOOLBAR_GROUP_HEIGHT: f32 = 28.0;
    const MENU_SEPARATOR_PADDING: f32 = 1.0;
    const SEPARATOR_PADDING: f32 = 9.0;
    const LABEL_LR_PADDING: f32 = 5.0;

    const ICON_FRAME_SIZE: f32 = 26.0;
    const ICON_SIZE: f32 = 25.0;
    const SMALL_ICON_FRAME_SIZE: f32 = 20.0;
    const SMALL_ICON_SIZE: f32 = 18.0;

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

                Element::new(cx)
                    .left(Pixels(MENU_SEPARATOR_PADDING))
                    .right(Pixels(MENU_SEPARATOR_PADDING))
                    .class("top_bar_separator");

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "Edit").class("menu_bar_label"),
                    |cx| {
                        MenuButton::new_simple(cx, "TODO", |_| {});
                    },
                );

                Element::new(cx)
                    .left(Pixels(MENU_SEPARATOR_PADDING))
                    .right(Pixels(MENU_SEPARATOR_PADDING))
                    .class("top_bar_separator");

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "View"),
                    |cx| {
                        MenuButton::new_simple(cx, "TODO", |_| {});
                    },
                );

                Element::new(cx)
                    .left(Pixels(MENU_SEPARATOR_PADDING))
                    .right(Pixels(MENU_SEPARATOR_PADDING))
                    .class("top_bar_separator");

                Menu::new(
                    cx,
                    |cx| Label::new(cx, "Help"),
                    |cx| {
                        MenuButton::new_simple(cx, "About", |_| {});
                    },
                );
            });
        })
        .top(Stretch(1.0))
        .bottom(Stretch(1.0));

        Element::new(cx)
            .left(Pixels(SEPARATOR_PADDING))
            .right(Pixels(SEPARATOR_PADDING))
            .class("top_bar_separator");

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Undo, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Redo, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");
        })
        .class("toolbar_group")
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Save, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");
        })
        .class("toolbar_group")
        .left(Pixels(SEPARATOR_PADDING))
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);

        Element::new(cx).left(Pixels(SEPARATOR_PADDING)).class("top_bar_separator");

        HStack::new(cx, |cx| {
            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Loop, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Stop, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Play, ICON_FRAME_SIZE, ICON_SIZE))
                .class("icon_btn");

            Element::new(cx).class("toolbar_group_separator");

            Button::new(
                cx,
                |_| {},
                |cx| Icon::new(cx, IconCode::Record, ICON_FRAME_SIZE, ICON_SIZE),
            )
            .class("record_btn");

            Element::new(cx).class("toolbar_group_separator");

            // TODO: Make this a functional widget.
            Label::new(cx, "1.1.1")
                .left(Pixels(35.0))
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .right(Pixels(LABEL_LR_PADDING));
        })
        .left(Stretch(1.0))
        .right(Stretch(1.0))
        .class("toolbar_group")
        .position_type(PositionType::SelfDirected)
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto);

        Element::new(cx)
            .right(Pixels(SEPARATOR_PADDING))
            .left(Stretch(1.0))
            .class("top_bar_separator");

        HStack::new(cx, |cx| {
            // TODO: Make this a functional widget.
            Label::new(cx, "BPM")
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Pixels(LABEL_LR_PADDING))
                .right(Pixels(LABEL_LR_PADDING))
                .class("toolbar_group_dimmed_label");

            //Element::new(cx).class("toolbar_group_separator");

            Label::new(cx, "120.000")
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Pixels(LABEL_LR_PADDING))
                .right(Pixels(LABEL_LR_PADDING));
        })
        .class("toolbar_group")
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .width(Auto)
        .right(Pixels(SEPARATOR_PADDING));

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "TAP")
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .right(Pixels(LABEL_LR_PADDING));
                    })
                    .child_left(Pixels(LABEL_LR_PADDING))
                    .child_right(Pixels(LABEL_LR_PADDING))
                },
            )
            .class("icon_btn")
            .top(Stretch(1.0))
            .bottom(Stretch(1.0))
            .height(Pixels(TOOLBAR_GROUP_HEIGHT - 2.0));
        })
        .width(Auto)
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .class("toolbar_group")
        .right(Pixels(SEPARATOR_PADDING));

        Element::new(cx).right(Pixels(SEPARATOR_PADDING)).class("top_bar_separator");

        HStack::new(cx, |cx| {
            Label::new(cx, "4 / 4")
                .class("time_signature_text")
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Pixels(LABEL_LR_PADDING))
                .right(Pixels(LABEL_LR_PADDING));
        })
        .width(Auto)
        .class("toolbar_group")
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .right(Pixels(SEPARATOR_PADDING));

        HStack::new(cx, |cx| {
            Button::new(
                cx,
                |_| {},
                |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "GRV")
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0))
                            .right(Pixels(LABEL_LR_PADDING));
                        Icon::new(cx, IconCode::Menu, SMALL_ICON_FRAME_SIZE, SMALL_ICON_SIZE)
                            .top(Stretch(1.0))
                            .bottom(Stretch(1.0));
                    })
                    .child_left(Pixels(LABEL_LR_PADDING))
                    .child_right(Pixels(LABEL_LR_PADDING))
                },
            )
            .class("icon_btn")
            .top(Stretch(1.0))
            .bottom(Stretch(1.0))
            .height(Pixels(TOOLBAR_GROUP_HEIGHT - 2.0));
        })
        .width(Auto)
        .height(Pixels(TOOLBAR_GROUP_HEIGHT))
        .class("toolbar_group")
        .right(Pixels(SEPARATOR_PADDING));
    })
    .height(Pixels(TOP_BAR_HEIGHT))
    .child_space(Pixels(TOP_BAR_CHILD_SPACE))
    .class("top_bar");
}
