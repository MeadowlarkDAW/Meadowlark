use std::path::PathBuf;
use vizia::prelude::*;

use crate::state_system::source_state::BrowserPanelTab;
use crate::state_system::{AppAction, BrowserPanelAction, SourceState};
use crate::ui::generic_views::virtual_slider::VirtualSliderState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserListEntryType {
    AudioFile,
    UnkownFile,
    Folder,
}

#[derive(Debug, Lens, Clone)]
pub struct BrowserListEntry {
    pub type_: BrowserListEntryType,
    pub name: String,
    pub selected: bool,

    #[lens(ignore)]
    pub path: PathBuf,
}

#[derive(Debug, Lens, Clone)]
pub struct BrowserPanelState {
    pub panel_shown: bool,
    pub current_tab: BrowserPanelTab,
    pub panel_width: f32,
    pub volume: VirtualSliderState,
    pub playback_on_select: bool,

    pub search_text: String,
    pub current_directory_text: String,
    pub list_entries: Vec<BrowserListEntry>,
    pub selected_entry_index: Option<usize>,

    #[lens(ignore)]
    pub root_sample_directories: Vec<PathBuf>,

    #[lens(ignore)]
    parent_subdirectories: Vec<PathBuf>,
}

impl BrowserPanelState {
    pub fn new(state: &SourceState) -> Self {
        let mut new_self = Self {
            panel_shown: state.app.browser_panel.panel_shown,
            current_tab: state.app.browser_panel.current_tab,
            panel_width: state.app.browser_panel.panel_width,
            volume: VirtualSliderState::from_value(
                state.app.browser_panel.volume_normalized,
                state.app.browser_panel.volume_default_normalized,
            ),
            playback_on_select: state.app.browser_panel.playback_on_select,

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
            self.list_entries.push(BrowserListEntry {
                type_: BrowserListEntryType::Folder,
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
                let mut directory_entries: Vec<BrowserListEntry> = Vec::new();
                let mut file_entries: Vec<BrowserListEntry> = Vec::new();

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
                                directory_entries.push(BrowserListEntry {
                                    type_: BrowserListEntryType::Folder,
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
                                                BrowserListEntryType::AudioFile
                                            }
                                            _ => BrowserListEntryType::UnkownFile,
                                        }
                                    } else {
                                        BrowserListEntryType::UnkownFile
                                    }
                                } else {
                                    BrowserListEntryType::UnkownFile
                                };

                                file_entries.push(BrowserListEntry {
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
                BrowserListEntryType::AudioFile => {
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
                BrowserListEntryType::UnkownFile => {
                    self.selected_entry_index = Some(index);
                    entry.selected = true;
                }
                BrowserListEntryType::Folder => {
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
