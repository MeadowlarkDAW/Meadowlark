use vizia::prelude::*;

use crate::state_system::{AppAction, BoundUiState, BrowserPanelTab, StateSystem};
use crate::ui::icon::{Icon, IconCode};
use crate::ui::views::resizable_stack::ResizableHStackDragR;

pub fn browser_panel(cx: &mut Context) {
    const ICON_FRAME_SIZE: f32 = 20.0;
    const ICON_SIZE: f32 = 18.0;
    const SEARCH_ICON_SIZE: f32 = 14.0;

    ResizableHStackDragR::new(
        cx,
        StateSystem::bound_ui_state.then(BoundUiState::browser_panel_width),
        |cx, width| {
            cx.emit(AppAction::SetBrowserPanelWidth(width));
        },
        |cx| {
            Label::new(cx, "BROWSER").class("small_text").bottom(Pixels(1.0));

            HStack::new(cx, |cx| {
                Textbox::new(
                    cx,
                    StateSystem::bound_ui_state.then(BoundUiState::browser_panel_search_text),
                )
                .on_edit(|cx, text| {
                    cx.emit(AppAction::SetBrowserPanelSearchText(text));
                })
                .width(Stretch(1.0))
                .height(Pixels(22.0));

                HStack::new(cx, |cx| {
                    Button::new(
                        cx,
                        |_| {},
                        |cx| Icon::new(cx, IconCode::Search, ICON_FRAME_SIZE, SEARCH_ICON_SIZE),
                    )
                    .class("icon_btn");

                    Element::new(cx).class("search_btn_group_separator");

                    Button::new(
                        cx,
                        |_| {},
                        |cx| Icon::new(cx, IconCode::Filter, ICON_FRAME_SIZE, SEARCH_ICON_SIZE),
                    )
                    .class("icon_btn");
                })
                .class("search_btn_group")
                .left(Pixels(8.0))
                .height(Auto)
                .width(Auto);
            })
            .height(Auto);

            VStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::Samples)),
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::Soundwave, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Samples").top(Stretch(1.0)).bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::Samples),
                );

                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::Multisamples)),
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::Piano, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Multisamples").top(Stretch(1.0)).bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::Multisamples),
                );

                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::Synths)),
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::Knob, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Synths").top(Stretch(1.0)).bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::Synths),
                );

                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::Effects)),
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::FX, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Effects").top(Stretch(1.0)).bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::Effects),
                );

                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::PianoRollClips)),
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::Midi, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Piano Roll Clips")
                                .top(Stretch(1.0))
                                .bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::PianoRollClips),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::AutomationClips))
                    },
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::Automation, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Automation Clips")
                                .top(Stretch(1.0))
                                .bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::AutomationClips),
                );

                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::Projects)),
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::FileAudio, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Projects").top(Stretch(1.0)).bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::Projects),
                );

                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::SelectBrowserPanelTab(BrowserPanelTab::Files)),
                    |cx| {
                        HStack::new(cx, |cx| {
                            Icon::new(cx, IconCode::Folder, ICON_FRAME_SIZE, ICON_SIZE);
                            Label::new(cx, "Files").top(Stretch(1.0)).bottom(Stretch(1.0));
                        })
                        .col_between(Pixels(4.0))
                    },
                )
                .class("browser_panel_tab")
                .toggle_class(
                    "browser_panel_tab_toggled",
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel_tab)
                        .map(|t| *t == BrowserPanelTab::Files),
                );
            })
            .height(Auto)
            .top(Pixels(6.0))
            .left(Pixels(6.0))
            .right(Pixels(6.0))
            .class("browser_panel_tabs");

            VStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Home, 26.0, 16.0))
                            .class("icon_btn");

                        Element::new(cx).class("search_btn_group_separator");

                        Button::new(
                            cx,
                            |_| {},
                            |cx| Icon::new(cx, IconCode::ChevronUp, 26.0, 20.0),
                        )
                        .class("icon_btn");

                        Element::new(cx).class("search_btn_group_separator");

                        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Undo, 26.0, 24.0))
                            .class("icon_btn");

                        Element::new(cx).class("search_btn_group_separator");

                        Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Redo, 26.0, 24.0))
                            .class("icon_btn");
                    });

                    Element::new(cx).class("browser_separator");

                    Label::new(cx, "../current-directory").class("small_text").left(Pixels(7.0));

                    Element::new(cx).class("browser_separator");
                })
                .height(Auto);

                ScrollView::new(cx, 0.0, 0.0, true, true, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "test-folder-1");
                        Label::new(cx, "test-folder-2");
                        Label::new(cx, "test-folder-3");
                        Label::new(cx, "test-folder-4");

                        Label::new(cx, "test-content-1.wav");
                        Label::new(cx, "test-content-2.wav");
                        Label::new(cx, "test-content-3.wav");
                        Label::new(cx, "test-content-4.wav");
                        Label::new(cx, "test-content-5.wav");
                        Label::new(cx, "test-content-6.wav");
                        Label::new(cx, "test-content-7.wav");
                        Label::new(cx, "test-content-8.wav");
                        Label::new(cx, "test-content-9.wav");
                        Label::new(cx, "test-content-10.wav");
                        Label::new(cx, "test-content-11.wav");
                        Label::new(cx, "test-content-12.wav");
                        Label::new(cx, "test-content-13.wav");
                        Label::new(cx, "test-content-14.wav");
                        Label::new(cx, "test-content-15.wav");
                        Label::new(cx, "test-content-16.wav");
                        Label::new(cx, "test-content-17.wav");
                        Label::new(cx, "test-content-18.wav");
                        Label::new(cx, "test-content-19.wav");
                        Label::new(cx, "test-content-20.wav");
                        Label::new(cx, "test-content-21.wav");
                        Label::new(cx, "test-content-22.wav");
                        Label::new(cx, "test-content-23.wav");
                        Label::new(cx, "test-content-24.wav");
                        Label::new(cx, "test-content-25.wav");
                    })
                    .child_left(Pixels(7.0))
                    .child_top(Pixels(2.0))
                    .child_bottom(Pixels(10.0))
                    .height(Stretch(1.0));
                })
                .height(Stretch(1.0));
            })
            .space(Pixels(6.0))
            .class("browser_panel_content");

            HStack::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Cursor, 24.0, 22.0))
                        .class("icon_btn");

                    Element::new(cx).class("search_btn_group_separator");

                    Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Play, 24.0, 22.0))
                        .class("icon_btn");

                    Element::new(cx).class("search_btn_group_separator");

                    Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Stop, 24.0, 22.0))
                        .class("icon_btn");

                    Element::new(cx).class("search_btn_group_separator");

                    Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Loop, 24.0, 22.0))
                        .class("icon_btn");
                })
                .class("search_btn_group")
                .height(Auto)
                .width(Auto);

                Knob::new(
                    cx,
                    0.75,
                    StateSystem::bound_ui_state.then(BoundUiState::browser_panel_volume_normalized),
                    false,
                )
                .class("browser_panel_knob")
                .on_changing(|cx, val| cx.emit(AppAction::SetBrowserVolumeNormalized(val)))
                .top(Stretch(1.0))
                .bottom(Stretch(1.0))
                .left(Pixels(8.0));
            })
            .width(Stretch(1.0))
            .height(Pixels(28.0));
        },
    )
    .height(Stretch(1.0))
    .child_space(Pixels(6.0))
    .class("browser_panel")
    .display(StateSystem::bound_ui_state.then(BoundUiState::browser_panel_shown));
}
