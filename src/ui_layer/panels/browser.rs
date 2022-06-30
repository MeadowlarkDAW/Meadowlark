use vizia::prelude::*;

use crate::{
    program_layer::{
        program_state::{BrowserEvent, BrowserState, File, PanelEvent, PanelState},
        ProgramLayer, ProgramState,
    },
    ui_layer::{Directory, Panel, ResizableStack},
};

// A simple file browser.
pub fn browser(cx: &mut Context) {
    // For testing purposes this event is emitted on browser creation to trigger the browser state to update.
    cx.emit(BrowserEvent::ViewAll);

    // A resizable stack so that the user can change the width of the browser panel.
    // Resizing the panel smaller than a certain size will collapse the panel (see panels state).
    ResizableStack::new(
        cx,
        ProgramLayer::state.then(ProgramState::panels.then(PanelState::browser_width)),
        |cx, width| {
            cx.emit(PanelEvent::SetBrowserWidth(width));
        },
        |cx| {
            // The actual browser panel
            Panel::new(
                cx,
                |cx| {
                    // Header
                    Label::new(cx, "BROWSER").class("small");
                },
                |cx| {
                    // Content
                    // The tree view of files in the browser in constructed recursively from the root file.
                    // Bind to the root file so that if it changes the tree view will be rebuilt.
                    Binding::new(
                        cx,
                        ProgramLayer::state
                            .then(ProgramState::browser.then(BrowserState::root_file)),
                        |cx, root_file| {
                            let root = root_file.get(cx);
                            // Recursively construct the tree view
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

// A view representing a directory or file in the browser.
fn directory(cx: &mut Context, name: &String, children: &Vec<File>, level: usize) {
    Directory::new(
        cx,
        |cx| {
            HStack::new(cx, |cx| {
                //Icon::new(cx, IconCode::Dropdown, 24.0, 23.0)
                Label::new(cx, "\u{e75c}")
                    .font("icon")
                    .height(Stretch(1.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .rotate(Directory::is_open.map(|flag| if *flag { 0.0 } else { -90.0 }));
                Label::new(cx, name).width(Stretch(1.0)).text_wrap(false);
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
