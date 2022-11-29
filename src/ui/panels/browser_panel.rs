use std::path::PathBuf;
use vizia::prelude::*;

use crate::state_system::app_state::BrowserPanelTab;
use crate::state_system::{AppAction, AppState, BoundUiState, BrowserPanelAction, StateSystem};
use crate::ui::generic_views::resizable_stack::ResizableHStackDragR;
use crate::ui::generic_views::{Icon, IconCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundBrowserListEntryType {
    AudioFile,
    UnkownFile,
    Folder,
}

#[derive(Debug, Lens, Clone)]
pub struct BoundBrowserListEntry {
    pub type_: BoundBrowserListEntryType,
    pub name: String,
    pub selected: bool,

    #[lens(ignore)]
    pub path: PathBuf,
}

#[derive(Debug, Lens, Clone)]
pub struct BoundBrowserPanelState {
    pub panel_shown: bool,
    pub current_tab: BrowserPanelTab,
    pub panel_width: f32,
    pub volume_normalized: f32,
    pub playback_on_select: bool,

    pub search_text: String,
    pub current_directory_text: String,
    pub list_entries: Vec<BoundBrowserListEntry>,
    pub selected_entry_index: Option<usize>,

    #[lens(ignore)]
    pub root_sample_directories: Vec<PathBuf>,

    #[lens(ignore)]
    parent_subdirectories: Vec<PathBuf>,
}

impl BoundBrowserPanelState {
    pub fn new(state: &AppState) -> Self {
        let mut new_self = Self {
            panel_shown: state.browser_panel.panel_shown,
            current_tab: state.browser_panel.current_tab,
            panel_width: state.browser_panel.panel_width,
            volume_normalized: state.browser_panel.volume_normalized,
            playback_on_select: state.browser_panel.playback_on_select,

            search_text: String::new(),
            current_directory_text: String::new(),
            list_entries: Vec::new(),
            selected_entry_index: None,
            root_sample_directories: vec!["./assets/test_files".into()],
            parent_subdirectories: Vec::new(),
        };

        match new_self.current_tab {
            BrowserPanelTab::Samples => new_self.enter_samples_root_directories(),
            _ => {}
        }

        new_self
    }

    pub fn enter_root_directory(&mut self) {
        match self.current_tab {
            BrowserPanelTab::Samples => {
                self.enter_samples_root_directories();
            }
            _ => {
                // TODO
            }
        }
    }

    fn enter_samples_root_directories(&mut self) {
        self.current_tab = BrowserPanelTab::Samples;
        self.current_directory_text.clear();
        self.selected_entry_index = None;
        self.list_entries.clear();
        self.parent_subdirectories.clear();

        for d in self.root_sample_directories.iter() {
            self.list_entries.push(BoundBrowserListEntry {
                type_: BoundBrowserListEntryType::Folder,
                name: d
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "<error>".into()),
                selected: false,
                path: d.clone(),
            });
        }
    }

    fn enter_subdirectory(&mut self, subdirectory_path: &PathBuf) {
        self.current_directory_text = subdirectory_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "<error>".into());

        self.populate_current_sudirectory();
    }

    fn populate_current_sudirectory(&mut self) {
        self.selected_entry_index = None;
        self.list_entries.clear();

        let current_subdirectory_path = self
            .parent_subdirectories
            .last()
            .expect("called `populate_current_sudirectory()` while in the root directory");

        match std::fs::read_dir(current_subdirectory_path) {
            Ok(reader) => {
                // We want to store directories before files, so use this intermediary vec.
                let mut directory_entries: Vec<BoundBrowserListEntry> = Vec::new();
                let mut file_entries: Vec<BoundBrowserListEntry> = Vec::new();

                for res in reader {
                    match res {
                        Ok(entry) => {
                            let file_type = match entry.file_type() {
                                Ok(t) => t,
                                Err(e) => {
                                    log::warn!("Failed to read item in directory: {}", e);
                                    continue;
                                }
                            };

                            if file_type.is_dir() {
                                directory_entries.push(BoundBrowserListEntry {
                                    type_: BoundBrowserListEntryType::Folder,
                                    name: entry.file_name().to_string_lossy().to_string(),
                                    selected: false,
                                    path: entry.path(), // We store the full path for directories.
                                });
                            } else if file_type.is_file() {
                                let type_ = if let Some(extension) = entry.path().extension() {
                                    if let Some(extension) = extension.to_str() {
                                        match extension.as_ref() {
                                            // TODO: More extensions
                                            "wav" | "mp3" | "flac" | "ogg" => {
                                                BoundBrowserListEntryType::AudioFile
                                            }
                                            _ => BoundBrowserListEntryType::UnkownFile,
                                        }
                                    } else {
                                        BoundBrowserListEntryType::UnkownFile
                                    }
                                } else {
                                    BoundBrowserListEntryType::UnkownFile
                                };

                                file_entries.push(BoundBrowserListEntry {
                                    type_,
                                    name: entry.file_name().to_string_lossy().to_string(),
                                    selected: false,
                                    path: entry.file_name().into(), // We only store the file name for files to save memory.
                                });
                            } // TODO: Support symlinks
                        }
                        Err(e) => {
                            log::warn!("Failed to read item in directory: {}", e);
                        }
                    }
                }

                // Sort items alphanumerically.
                directory_entries.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));
                file_entries.sort_by(|a, b| alphanumeric_sort::compare_str(&a.name, &b.name));

                self.list_entries.append(&mut directory_entries);
                self.list_entries.append(&mut file_entries);
            }
            Err(e) => {
                log::error!("Couldn't read subdirectory {:?}: {}", &current_subdirectory_path, e);
            }
        }
    }

    pub fn enter_parent_directory(&mut self) {
        match self.parent_subdirectories.pop() {
            Some(_current_directory) => {}
            None => {
                // Already at the root directory.
                return;
            }
        };

        let mut enter_subdirectory = None;
        match self.parent_subdirectories.last() {
            Some(parent_directory) => {
                enter_subdirectory = Some(parent_directory.clone());
            }
            None => self.enter_root_directory(),
        }

        if let Some(directory) = enter_subdirectory {
            self.enter_subdirectory(&directory);
        }
    }

    pub fn refresh(&mut self) {
        let mut enter_subdirectory = None;
        match self.parent_subdirectories.last() {
            Some(parent_directory) => {
                enter_subdirectory = Some(parent_directory.clone());
            }
            None => self.enter_root_directory(),
        }

        if let Some(directory) = enter_subdirectory {
            self.enter_subdirectory(&directory);
        }
    }

    pub fn select_entry_by_index(
        &mut self,
        cx: &mut EventContext,
        index: usize,
        invoked_by_play_btn: bool,
    ) {
        if let Some(old_entry_i) = self.selected_entry_index.take() {
            if let Some(old_entry) = &mut self.list_entries.get_mut(old_entry_i) {
                old_entry.selected = false;
            }
        }

        let mut enter_subdirectory = None;
        if let Some(entry) = self.list_entries.get_mut(index) {
            match entry.type_ {
                BoundBrowserListEntryType::AudioFile => {
                    self.selected_entry_index = Some(index);
                    entry.selected = true;

                    if self.playback_on_select || invoked_by_play_btn {
                        if let Some(parent_directory) = self.parent_subdirectories.last() {
                            let mut path = parent_directory.clone();
                            path.push(&entry.path);

                            cx.emit(AppAction::BrowserPanel(BrowserPanelAction::PlayFile(path)));
                        }
                    }
                }
                BoundBrowserListEntryType::UnkownFile => {
                    self.selected_entry_index = Some(index);
                    entry.selected = true;
                }
                BoundBrowserListEntryType::Folder => {
                    enter_subdirectory = Some(entry.path.clone());
                }
            }
        }

        if let Some(directory) = enter_subdirectory {
            self.parent_subdirectories.push(directory.clone());

            self.enter_subdirectory(&directory);
        }
    }
}

impl Model for BoundBrowserPanelState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}
}

pub fn browser_panel(cx: &mut Context) {
    const ICON_FRAME_SIZE: f32 = 20.0;
    const ICON_SIZE: f32 = 18.0;
    const SEARCH_ICON_SIZE: f32 = 14.0;

    ResizableHStackDragR::new(
        cx,
        StateSystem::bound_ui_state
            .then(BoundUiState::browser_panel)
            .then(BoundBrowserPanelState::panel_width),
        |cx, width| {
            cx.emit(AppAction::BrowserPanel(BrowserPanelAction::SetPanelWidth(width)));
        },
        |cx| {
            Label::new(cx, "BROWSER").class("small_text").bottom(Pixels(1.0));

            HStack::new(cx, |cx| {
                Textbox::new(
                    cx,
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::search_text),
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                    StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::current_tab)
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
                StateSystem::bound_ui_state
                    .then(BoundUiState::browser_panel)
                    .then(BoundBrowserPanelState::current_tab),
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
        StateSystem::bound_ui_state
            .then(BoundUiState::browser_panel)
            .then(BoundBrowserPanelState::panel_shown),
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
                StateSystem::bound_ui_state
                    .then(BoundUiState::browser_panel)
                    .then(BoundBrowserPanelState::current_directory_text),
            )
            .class("small_text")
            .left(Pixels(7.0));

            Element::new(cx).class("browser_separator");
        })
        .height(Auto);

        ScrollView::new(cx, 0.0, 0.0, true, true, |cx| {
            List::new(
                cx,
                StateSystem::bound_ui_state
                    .then(BoundUiState::browser_panel)
                    .then(BoundBrowserPanelState::list_entries),
                |cx, index, entry| {
                    Button::new(
                        cx,
                        |_| {},
                        |cx| {
                            HStack::new(cx, |cx| {
                                Icon::new(
                                    cx,
                                    entry.map(|e| match e.type_ {
                                        BoundBrowserListEntryType::AudioFile => IconCode::Soundwave,
                                        BoundBrowserListEntryType::UnkownFile => IconCode::File,
                                        BoundBrowserListEntryType::Folder => IconCode::Folder,
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
                        !StateSystem::bound_ui_state
                            .then(BoundUiState::browser_panel)
                            .then(BoundBrowserPanelState::playback_on_select)
                            .get(cx),
                    )))
                },
                |cx| Icon::new(cx, IconCode::Cursor, 24.0, 22.0),
            )
            .toggle_class(
                "icon_btn_accent_toggled",
                StateSystem::bound_ui_state
                    .then(BoundUiState::browser_panel)
                    .then(BoundBrowserPanelState::playback_on_select),
            )
            .class("icon_btn");

            Element::new(cx).class("search_btn_group_separator");

            Button::new(cx, |_| {}, |cx| Icon::new(cx, IconCode::Play, 24.0, 22.0))
                .on_press_down(|cx| {
                    if let Some(index) = StateSystem::bound_ui_state
                        .then(BoundUiState::browser_panel)
                        .then(BoundBrowserPanelState::selected_entry_index)
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

        Knob::new(
            cx,
            1.0,
            StateSystem::bound_ui_state
                .then(BoundUiState::browser_panel)
                .then(BoundBrowserPanelState::volume_normalized),
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
    })
    .width(Stretch(1.0))
    .height(Pixels(28.0));
}
