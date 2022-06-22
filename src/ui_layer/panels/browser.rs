use vizia::prelude::*;

use crate::{
    program_layer::{
        program_state::{BrowserEvent, BrowserState, File, PanelEvent, PanelState},
        ProgramLayer, ProgramState,
    },
    ui_layer::{Directory, Panel, ResizableStack},
};

pub fn browser(cx: &mut Context) {
    cx.emit(BrowserEvent::ViewAll);

    ResizableStack::new(
        cx,
        ProgramLayer::state.then(ProgramState::panels.then(PanelState::browser_width)),
        |cx, width| {
            cx.emit(PanelEvent::SetBrowserWidth(width));
        },
        |cx| {
            Panel::new(
                cx,
                |cx| {
                    Label::new(cx, "BROWSER").class("small");
                },
                |cx| {
                    Binding::new(
                        cx,
                        ProgramLayer::state
                            .then(ProgramState::browser.then(BrowserState::root_file)),
                        |cx, root_file| {
                            let root = root_file.get(cx);

                            directory(cx, &root.name, &root.children, 0);
                        },
                    )
                },
            );
        },
    )
    .class("browser")
    .display(ProgramLayer::state.then(ProgramState::panels.then(PanelState::show_browser)));
}

// A view representing a directory in the browser
fn directory(cx: &mut Context, name: &String, children: &Vec<File>, level: usize) {
    Directory::new(
        cx,
        |cx| {
            HStack::new(cx, |cx|{
                //Icon::new(cx, IconCode::Dropdown, 24.0, 23.0)
                Label::new(cx, "\u{e75c}").font("icon")
                    .height(Stretch(1.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .rotate(Directory::is_open.map(|flag| if *flag {0.0} else {-90.0}));
                Label::new(cx, name)
                    .width(Stretch(1.0))
                    .text_wrap(false);    
            })
            .class("dir-file")
            .col_between(Pixels(4.0))
            .child_left(Pixels(15.0 * level as f32 + 5.0));
        },
        |cx| {
            for file in children.iter() {
                if file.children.is_empty() {
                    Label::new(cx, &file.name)
                        .class("dir-file")
                        .width(Stretch(1.0))
                        .text_wrap(false)
                        .child_left(Pixels(15.0 * (level + 1) as f32 + 5.0));
                } else {
                    directory(cx, &file.name, &file.children, level + 1);
                }
            }
        },
    );
}

// A view representing a file in the browser
fn file(cx: &mut Context) {}
