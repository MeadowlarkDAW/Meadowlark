use vizia::prelude::*;

use crate::ui::icon::{Icon, IconCode};

static FONT_FAMILY: &str = "min-sans-medium";
static FONT_SIZE: f32 = 12.0;

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
    })
    .class("top_bar");
}
