use std::path::{Path, PathBuf};

use vizia::prelude::*;

use crate::ui::state::{BrowserEvent, BrowserState, File, PanelEvent, PanelState};
use crate::ui::{Directory, DirectoryEvent, Panel, ResizableStack, UiData, UiEvent, UiState};

// A simple file browser.
pub fn browser(cx: &mut Context) {
    // For testing purposes this event is emitted on browser creation to trigger the browser state to update.
    cx.emit(BrowserEvent::ViewAll);

    // A resizable stack so that the user can change the width of the browser panel.
    // Resizing the panel smaller than a certain size will collapse the panel (see panels state).
    ResizableStack::new(
        cx,
        UiData::state.then(UiState::panels.then(PanelState::browser_width)),
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
                        UiData::state.then(UiState::browser.then(BrowserState::root_file)),
                        |cx, root_file| {
                            let root = root_file.get(cx);
                            // Recursively construct the tree view
                            directory(cx, &root.name, &root.file_path, &root.children, 0);
                        },
                    )
                },
            );
        },
    )
    .class("browser")
    .display(UiData::state.then(UiState::panels.then(PanelState::show_browser)));
}

// A view representing a directory or file in the browser.
fn directory(
    cx: &mut Context,
    name: &String,
    file_path: &Option<PathBuf>,
    children: &Vec<File>,
    level: usize,
) {
    let path = file_path.clone();
    Directory::new(
        cx,
        file_path,
        |cx| {
            let file_path1 = file_path.clone();
            let file_path2 = file_path.clone();
            let file_path3 = file_path.clone();
            HStack::new(cx, |cx| {
                //Icon::new(cx, IconCode::Dropdown, 24.0, 23.0)
                // Arrow Icon
                Label::new(cx, "\u{e75c}")
                    .font("icon")
                    .height(Stretch(1.0))
                    .child_top(Stretch(1.0))
                    .child_bottom(Stretch(1.0))
                    .rotate(Directory::is_open.map(|flag| if *flag { 0.0 } else { -90.0 }));
                // File or directory name
                Label::new(cx, name).width(Stretch(1.0)).text_wrap(false);
            })
            .class("dir-file")
            .toggle_class(
                "focused",
                UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                    move |selected| {
                        if let Some(path) = &file_path1 {
                            selected.starts_with(path)
                        } else {
                            false
                        }
                    },
                ))),
            )
            .toggle_class(
                "selected",
                UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                    move |selected| {
                        if let Some(path) = &file_path2 {
                            selected == path
                        } else {
                            false
                        }
                    },
                ))),
            )
            .on_press(move |cx| {
                if let Some(file_path) = &file_path3 {
                    cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                }
            })
            .col_between(Pixels(4.0))
            .child_left(Pixels(15.0 * level as f32 + 5.0));
        },
        |cx| {
            let file_path1 = file_path.clone();
            for file in children.iter() {
                if file.children.is_empty() {
                    let file_path1 = file.file_path.clone();
                    let file_path2 = file.file_path.clone();
                    let file_path3 = file.file_path.clone();
                    Label::new(cx, &file.name)
                        .class("dir-file")
                        .width(Stretch(1.0))
                        .text_wrap(false)
                        .child_left(Pixels(15.0 * (level + 1) as f32 + 5.0))
                        .toggle_class(
                            "focused",
                            UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                                move |selected| {
                                    if let Some(path) = &file_path1 {
                                        selected.starts_with(path)
                                    } else {
                                        false
                                    }
                                },
                            ))),
                        )
                        .toggle_class(
                            "selected",
                            UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                                move |selected| {
                                    if let Some(path) = &file_path2 {
                                        selected == path
                                    } else {
                                        false
                                    }
                                },
                            ))),
                        )
                        .on_press(move |cx| {
                            if let Some(file_path) = &file_path3 {
                                cx.emit(UiEvent::BrowserFileClicked(file_path.clone()));
                                cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                            }
                        });
                } else {
                    directory(cx, &file.name, &file.file_path, &file.children, level + 1);
                }
            }

            Element::new(cx)
                .left(Pixels(15.0 * (level + 1) as f32 - 5.0))
                .height(Stretch(1.0))
                .width(Pixels(1.0))
                .position_type(PositionType::SelfDirected)
                .display(Directory::is_open)
                .class("dir-line")
                .toggle_class(
                    "focused",
                    UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                        move |selected| {
                            if let Some(path) = &file_path1 {
                                if let Some(dir) = dir_path(selected) {
                                    dir == path
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        },
                    ))),
                );
        },
    );
}

fn dir_path(path: &Path) -> Option<&Path> {
    if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    }
}
