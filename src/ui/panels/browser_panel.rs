use vizia::prelude::*;

use crate::state_system::source_state::BrowserPanelTab;
use crate::state_system::working_state::browser_panel_state::{
    BrowserListEntryType, BrowserPanelState,
};
use crate::state_system::{AppAction, BrowserPanelAction, StateSystem, WorkingState};
use crate::ui::generic_views::knob::{KnobView, KnobViewStyle};
use crate::ui::generic_views::resizable_stack::ResizableHStackDragR;
use crate::ui::generic_views::virtual_slider::{
    VirtualSliderDirection, VirtualSliderEvent, VirtualSliderMode, VirtualSliderScalars,
};
use crate::ui::generic_views::{Icon, IconCode};

pub fn browser_panel(cx: &mut Context) {
    const ICON_FRAME_SIZE: f32 = 20.0;
    const ICON_SIZE: f32 = 18.0;
    const SEARCH_ICON_SIZE: f32 = 14.0;

    ResizableHStackDragR::new(
        cx,
        StateSystem::working_state
            .then(WorkingState::browser_panel_lens)
            .then(BrowserPanelState::panel_width),
        |cx, width| {
            cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SetPanelWidth(width)));
        },
        |cx| {
            Label::new(cx, "BROWSER").class("small_text").bottom(Pixels(1.0));

            HStack::new(cx, |cx| {
                Textbox::new(
                    cx,
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::search_text),
                )
                .on_edit(|cx, text| {
                    cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SetSearchText(text)));
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
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::Samples,
                        )))
                    },
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::Samples),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::Multisamples,
                        )))
                    },
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::Multisamples),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::Synths,
                        )))
                    },
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::Synths),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::Effects,
                        )))
                    },
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::Effects),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::PianoRollClips,
                        )))
                    },
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::PianoRollClips),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::AutomationClips,
                        )))
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::AutomationClips),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::Projects,
                        )))
                    },
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::Projects),
                );

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectTab(
                            BrowserPanelTab::Files,
                        )))
                    },
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
                    StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::current_tab)
                        .map(|t| *t == BrowserPanelTab::Files),
                );
            })
            .height(Auto)
            .top(Pixels(6.0))
            .left(Pixels(6.0))
            .right(Pixels(6.0))
            .class("browser_panel_tabs");

            Binding::new(
                cx,
                StateSystem::working_state
                    .then(WorkingState::browser_panel_lens)
                    .then(BrowserPanelState::current_tab),
                |cx, tab| match tab.get(cx) {
                    BrowserPanelTab::Samples => browser_list(cx),
                    _ => {
                        Label::new(cx, "Not yet implemented").top(Pixels(2.0));
                    }
                },
            );
        },
    )
    .height(Stretch(1.0))
    .child_space(Pixels(6.0))
    .class("browser_panel")
    .display(
        StateSystem::working_state
            .then(WorkingState::browser_panel_lens)
            .then(BrowserPanelState::panel_shown),
    );
}

fn browser_list(cx: &mut Context) {
    VStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            HStack::new(cx, |cx| {
                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::EnterRootDirectory));
                    },
                    |cx| Icon::new(cx, IconCode::Home, 26.0, 16.0),
                )
                .class("icon_btn");

                Element::new(cx).class("search_btn_group_separator");

                Button::new(
                    cx,
                    |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::EnterParentDirectory));
                    },
                    |cx| Icon::new(cx, IconCode::ChevronUp, 26.0, 20.0),
                )
                .class("icon_btn");

                Element::new(cx).class("search_btn_group_separator");

                Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Undo, 26.0, 24.0))
                    .class("icon_btn");

                Element::new(cx).class("search_btn_group_separator");

                Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Redo, 26.0, 24.0))
                    .class("icon_btn");

                Button::new(
                    cx,
                    |cx| cx.emit(AppAction::BrowserPanel(BrowserPanelAction::Refresh)),
                    |cx| Icon::new(cx, IconCode::Refresh, 26.0, 16.0),
                )
                .class("icon_btn")
                .left(Stretch(1.0));

                Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Filter, 26.0, 16.0))
                    .class("icon_btn");
            });

            Element::new(cx).class("browser_separator");

            Label::new(
                cx,
                StateSystem::working_state
                    .then(WorkingState::browser_panel_lens)
                    .then(BrowserPanelState::current_directory_text),
            )
            .class("small_text")
            .left(Pixels(7.0));

            Element::new(cx).class("browser_separator");
        })
        .height(Auto);

        ScrollView::new(cx, 0.0, 0.0, true, true, |cx| {
            List::new(
                cx,
                StateSystem::working_state
                    .then(WorkingState::browser_panel_lens)
                    .then(BrowserPanelState::list_entries),
                |cx, index, entry| {
                    Button::new(
                        cx,
                        |_| {},
                        |cx| {
                            HStack::new(cx, |cx| {
                                Icon::new(
                                    cx,
                                    entry.map(|e| match e.type_ {
                                        BrowserListEntryType::AudioFile => IconCode::Soundwave,
                                        BrowserListEntryType::UnkownFile => IconCode::File,
                                        BrowserListEntryType::Folder => IconCode::Folder,
                                    }),
                                    20.0,
                                    16.0,
                                )
                                .left(Pixels(7.0))
                                .top(Stretch(1.0))
                                .bottom(Stretch(1.0));

                                Label::new(cx, entry.map(|e| e.name.clone()))
                                    .left(Pixels(3.0))
                                    .top(Stretch(1.0))
                                    .bottom(Stretch(1.0));
                            })
                        },
                    )
                    .height(Pixels(23.0))
                    .class("browser_entry")
                    .toggle_class("browser_entry_checked", entry.map(|e| e.selected))
                    .on_press_down(move |cx| {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectEntryByIndex {
                            index,
                            invoked_by_play_btn: false,
                        }))
                    });
                },
            )
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
            Button::new(
                cx,
                |cx| {
                    cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SetPlaybackOnSelect(
                        !StateSystem::working_state
                            .then(WorkingState::browser_panel_lens)
                            .then(BrowserPanelState::playback_on_select)
                            .get(cx),
                    )))
                },
                |cx| Icon::new(cx, IconCode::Cursor, 24.0, 22.0),
            )
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::working_state
                    .then(WorkingState::browser_panel_lens)
                    .then(BrowserPanelState::playback_on_select),
            )
            .class("icon_btn");

            Element::new(cx).class("search_btn_group_separator");

            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Play, 24.0, 22.0))
                .on_press_down(|cx| {
                    if let Some(index) = StateSystem::working_state
                        .then(WorkingState::browser_panel_lens)
                        .then(BrowserPanelState::selected_entry_index)
                        .get(cx)
                    {
                        cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SelectEntryByIndex {
                            index,
                            invoked_by_play_btn: true,
                        }));
                    }
                })
                .class("icon_btn");

            Element::new(cx).class("search_btn_group_separator");

            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Stop, 24.0, 22.0))
                .on_press_down(|cx| {
                    cx.emit(AppAction::BrowserPanel(BrowserPanelAction::StopPlayback));
                })
                .class("icon_btn");
        })
        .class("search_btn_group")
        .height(Auto)
        .width(Auto);

        KnobView::new(
            cx,
            StateSystem::working_state
                .then(WorkingState::browser_panel_lens)
                .then(BrowserPanelState::volume),
            VirtualSliderMode::Continuous,
            VirtualSliderDirection::Vertical,
            VirtualSliderScalars::default(),
            Pixels(9.0),
            false,
            KnobViewStyle::default(),
            |cx, event| match event {
                VirtualSliderEvent::Changed(value_normalized) => cx.emit(AppAction::BrowserPanel(
                    BrowserPanelAction::SetVolumeNormalized(value_normalized),
                )),
                _ => {}
            },
        )
        .top(Stretch(1.0))
        .bottom(Stretch(1.0))
        .width(Pixels(28.0))
        .height(Pixels(28.0))
        .left(Pixels(8.0));

        /*
        Knob::new(
            cx,
            1.0,
            StateSystem::working_state
                .then(WorkingState::browser_panel_lens)
                .then(BrowserPanelState::volume_normalized),
            false,
        )
        .class("browser_panel_knob")
        .on_changing(|cx, val_normalized| {
            cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SetVolumeNormalized(
                val_normalized,
            )))
        })
        .top(Stretch(1.0))
        .bottom(Stretch(1.0))
        .left(Pixels(8.0));
        */
    })
    .width(Stretch(1.0))
    .height(Pixels(28.0));
}
