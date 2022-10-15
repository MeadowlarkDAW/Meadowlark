use vizia::prelude::*;

use crate::state_system::{AppEvent, BoundUiState, BrowserPanelTab, StateSystem};
use crate::ui::icon::{Icon, IconCode};

pub fn browser_panel(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "BROWSER").class("small_text");

        VStack::new(cx, |cx| {
            Button::new(
                cx,
                |cx| cx.emit(AppEvent::SelectBrowserPanelTab(BrowserPanelTab::Samples)),
                |cx| {
                    HStack::new(cx, |cx| {
                        Icon::new(cx, IconCode::Audio, 20.0, 18.0);
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
                        Icon::new(cx, IconCode::Instrument, 20.0, 18.0);
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
                        Icon::new(cx, IconCode::Synth, 20.0, 18.0);
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
                        Icon::new(cx, IconCode::FX, 20.0, 18.0);
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
                        Icon::new(cx, IconCode::Midi, 20.0, 18.0);
                        Label::new(cx, "Piano Roll Clips").top(Stretch(1.0)).bottom(Stretch(1.0));
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
                        Icon::new(cx, IconCode::Automation, 20.0, 18.0);
                        Label::new(cx, "Automation Clips").top(Stretch(1.0)).bottom(Stretch(1.0));
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
                        Icon::new(cx, IconCode::Music, 20.0, 18.0);
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
                        Icon::new(cx, IconCode::Folder, 20.0, 18.0);
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
        .class("browser_panel_tabs");
    })
    .height(Stretch(1.0))
    .row_between(Pixels(5.0))
    .child_space(Pixels(4.0))
    .class("browser_panel")
    .display(StateSystem::bound_ui_state.then(BoundUiState::browser_panel_shown));
}
