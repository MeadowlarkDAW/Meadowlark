use std::path::Path;
use std::rc::Rc;

use vizia::prelude::*;

mod keymap;
use keymap::*;
use vizia::state::{Index, Then};

use crate::ui::file_derived_lenses::children;
use crate::ui::state::{BrowserEvent, BrowserState, File, PanelEvent, PanelState};
use crate::ui::{Panel, ResizableStack, UiData, UiEvent, UiState};

// A simple file browser.
pub fn browser(cx: &mut Context) {
    // For testing purposes this event is emitted on browser creation to trigger the browser state to update.
    cx.emit(BrowserEvent::ViewAll);

    browser_keymap(cx);

    HStack::new(cx, |cx| {
        // Placeholder for Left Bar
        VStack::new(cx, |cx| {
            Element::new(cx).class("level4").size(Pixels(32.0)).bottom(Pixels(1.0));

            Element::new(cx).class("level2").size(Pixels(32.0));

            Element::new(cx).class("level3").size(Pixels(32.0));

            Element::new(cx).class("level2").size(Pixels(32.0));

            Element::new(cx).class("level2").size(Pixels(32.0));

            Element::new(cx).class("level2").size(Pixels(32.0));
        })
        .width(Pixels(32.0))
        .class("level2");

        // Browser
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
                        Label::new(cx, "BROWSER").text_wrap(false).class("small");
                        Label::new(cx, "BROWSE2").on_release(|cx| {
                            if let Some(folder_path) = rfd::FileDialog::new().pick_folder() {
                                cx.emit(BrowserEvent::SetRootPath(folder_path.clone()));
                            }
                        });
                    },
                    |cx| {
                        // The tree view of files in the browser in constructed recursively from the root file.
                        // Bind to the root file so that if it changes the tree view will be rebuilt.
                        // TODO: Add more levels for the treeview
                        ScrollView::new(cx, 0.0, 0.0, false, false, |cx| {
                            treeview(
                                cx,
                                UiData::state.then(UiState::browser.then(BrowserState::root_file)),
                                0,
                                directory_header,
                                |cx, item, level| {
                                    treeview(
                                        cx,
                                        item,
                                        level,
                                        directory_header,
                                        |cx, item, level| {
                                            treeview(
                                                cx,
                                                item,
                                                level,
                                                directory_header,
                                                |cx, item, level| {
                                                    treeview(
                                                        cx,
                                                        item,
                                                        level,
                                                        directory_header,
                                                        |cx, item, level| {
                                                            treeview(
                                                                cx,
                                                                item,
                                                                level,
                                                                directory_header,
                                                                file,
                                                            );
                                                        },
                                                    );
                                                },
                                            );
                                        },
                                    );
                                },
                            );
                        })
                        .class("level3");
                    },
                )
                .display(
                    UiData::state
                        .then(UiState::panels.then(PanelState::hide_browser.map(|flag| !flag))),
                );
            },
        )
        .class("browser")
        .toggle_class("hidden", UiData::state.then(UiState::panels.then(PanelState::hide_browser)));
    })
    .width(Auto)
    .class("level1");
}

fn directory_header<L>(cx: &mut Context, root: L, level: u32)
where
    L: Lens<Target = File>,
{
    Binding::new(
        cx,
        root.clone().then(File::children).map(|items| items.len()),
        move |cx, num_items| {
            let num_children = num_items.get(cx);
            if num_children == 0 {
                file(cx, root.clone(), level);
            } else {
                directory(cx, root.clone(), level);
            }
        },
    );
}

fn directory<L>(cx: &mut Context, root: L, level: u32)
where
    L: Lens<Target = File>,
{
    Binding::new(cx, root.clone().then(File::file_path), move |cx, file_path| {
        let file_path1 = file_path.get(cx);
        let file_path2 = file_path.get(cx);
        let file_path3 = file_path.get(cx);
        HStack::new(cx, |cx| {
            //Icon::new(cx, IconCode::Dropdown, 24.0, 23.0)
            // Arrow Icon
            Label::new(cx, "\u{e75c}")
                .font("icon")
                .height(Stretch(1.0))
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .hoverable(false)
                .rotate(
                    root.clone().then(File::is_open).map(|flag| if *flag { 0.0 } else { -90.0 }),
                );
            // File or directory name
            Label::new(cx, root.clone().then(File::name))
                .width(Stretch(1.0))
                .text_wrap(false)
                .hoverable(false);
        })
        .cursor(CursorIcon::Hand)
        .class("dir-file")
        .toggle_class(
            "focused",
            UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                move |selected| match (&file_path1, selected) {
                    (Some(fp), Some(s)) => s.starts_with(fp),

                    _ => false,
                },
            ))),
        )
        .toggle_class(
            "selected",
            UiData::state.then(
                UiState::browser
                    .then(BrowserState::selected.map(move |selected| &file_path2 == selected)),
            ),
        )
        .on_press(move |cx| {
            cx.focus();
            if let Some(file_path) = &file_path3 {
                cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                cx.emit(BrowserEvent::ToggleOpen);
            }
        })
        .col_between(Pixels(4.0))
        .child_left(Pixels(15.0 * level as f32 + 5.0));
    });
}

fn file<L>(cx: &mut Context, item: L, level: u32)
where
    L: Lens<Target = File>,
{
    Binding::new(cx, item.clone().then(File::file_path), move |cx, file_path| {
        let file_path1 = file_path.get(cx);
        let file_path2 = file_path.get(cx);
        let file_path3 = file_path.get(cx);
        Label::new(cx, item.clone().then(File::name))
            .class("dir-file")
            .width(Stretch(1.0))
            .text_wrap(false)
            .cursor(CursorIcon::Hand)
            .child_left(Pixels(15.0 * level as f32 + 5.0))
            .toggle_class(
                "focused",
                UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                    move |selected| match (&file_path1, selected) {
                        (Some(fp), Some(s)) => s.starts_with(fp),
                        _ => false,
                    },
                ))),
            )
            .toggle_class(
                "selected",
                UiData::state.then(
                    UiState::browser
                        .then(BrowserState::selected.map(move |selected| &file_path2 == selected)),
                ),
            )
            .on_press(move |cx| {
                cx.focus();
                if let Some(file_path) = &file_path3 {
                    cx.emit(UiEvent::BrowserFileClicked(file_path.clone()));
                    cx.emit(BrowserEvent::SetSelected(file_path.clone()));
                }
            });
    });
}

fn treeview<L>(
    cx: &mut Context,
    lens: L,
    level: u32,
    header: impl Fn(&mut Context, L, u32),
    content: impl Fn(&mut Context, Then<Then<L, children>, Index<Vec<File>, File>>, u32) + 'static,
) where
    L: Lens<Target = File>,
    L::Source: Model,
{
    let content = Rc::new(content);
    VStack::new(cx, |cx| {
        // Label::new(cx, lens.clone().then(File::name));
        (header)(cx, lens.clone(), level);
        Binding::new(cx, lens.clone().then(File::is_open), move |cx, is_open| {
            if is_open.get(cx) {
                let content1 = content.clone();
                let lens2 = lens.clone();
                VStack::new(cx, |cx| {
                    List::new(cx, lens2.clone().then(File::children), move |cx, index, item| {
                        (content1.clone())(cx, item.clone(), level + 1);
                    })
                    .height(Auto);

                    let file_path1 = lens2.clone().get(cx).file_path.clone();
                    Element::new(cx)
                        .left(Pixels(15.0 * (level + 1) as f32 - 5.0))
                        .height(Stretch(1.0))
                        .width(Pixels(1.0))
                        .position_type(PositionType::SelfDirected)
                        .display(lens2.then(File::is_open))
                        .class("dir-line")
                        .toggle_class(
                            "focused",
                            UiData::state.then(UiState::browser.then(BrowserState::selected.map(
                                move |selected| {
                                    if let Some(path) = &file_path1 {
                                        if let Some(selected) = selected {
                                            if let Some(dir) = dir_path(selected) {
                                                dir == path
                                            } else {
                                                false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                },
                            ))),
                        );
                })
                .height(Auto);
                //.display(root.clone().then(File::is_open));
            }
        });
    })
    .height(Auto);
}

fn dir_path(path: &Path) -> Option<&Path> {
    if path.is_dir() {
        Some(path)
    } else {
        path.parent()
    }
}
