use std::path::Path;
use std::rc::Rc;

use vizia::prelude::*;

mod keymap;
use keymap::*;
use vizia::state::{Index, Then};

use crate::ui::file_derived_lenses::children;
use crate::ui::state::{BrowserEvent, BrowserState, File, PanelEvent, PanelState};
use crate::ui::{Panel, ResizableStack, UiData, UiEvent, UiState};

// A simple tooltip for the file browser.
pub fn tooltip(cx: &mut Context) {
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

