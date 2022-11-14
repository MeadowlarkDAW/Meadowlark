use vizia::prelude::*;

use crate::state_system::{AppEvent, BoundUiState, BrowserPanelTab, StateSystem};
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
            cx.emit(AppEvent::SetBrowserPanelWidth(width));
        },
        |cx| {
            Label::new(cx, "BROWSER").class("small_text").bottom(Pixels(1.0));

            HStack::new(cx, |cx| {
                Textbox::new(
                    cx,
                    StateSystem::bound_ui_state.then(BoundUiState::browser_panel_search_text),
                )
                .on_edit(|cx, text| {
                    cx.emit(AppEvent::SetBrowserPanelSearchText(text));
                })
                .width(Stretch(1.0))
                .height(Pixels(22.0));

                Icon::new(cx, IconCode::Search, ICON_FRAME_SIZE, SEARCH_ICON_SIZE)
                    .left(Pixels(3.0));
            })
            .height(Pixels(28.0));

            VStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::Samples)),
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
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::Multisamples)),
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
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::Synths)),
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
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::Effects)),
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
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::PianoRollClips)),
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
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::AutomationClips)),
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
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::Projects)),
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
                    |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::Files)),
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
            .space(Pixels(6.0))
            .class("browser_panel_tabs");

            Element::new(cx).top(Stretch(1.0)).bottom(Stretch(1.0));
        },
    )
    .height(Stretch(1.0))
    .row_between(Pixels(5.0))
    .child_space(Pixels(6.0))
    .class("browser_panel")
    .display(StateSystem::bound_ui_state.then(BoundUiState::browser_panel_shown));
}
