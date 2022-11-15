use vizia::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Data)]
pub enum BrowserPanelTab {
    Samples,
    Multisamples,
    Synths,
    Effects,
    PianoRollClips,
    AutomationClips,
    Projects,
    Files,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserListEntryType {
    AudioFile,
    Folder,
}

#[derive(Debug, Lens, Clone)]
pub struct BrowserListEntry {
    pub type_: BrowserListEntryType,
    pub text: String,
    pub selected: bool,
}

#[derive(Debug, Lens, Clone)]
pub struct BoundUiState {
    pub browser_panel_shown: bool,
    pub browser_panel_tab: BrowserPanelTab,
    pub browser_panel_width: f32,
    pub browser_panel_search_text: String,
    pub browser_panel_volume_normalized: f32,
    pub browser_current_directory: String,
    pub browser_list_entries: Vec<BrowserListEntry>,
    pub selected_browser_entry: Option<usize>,
}

impl BoundUiState {
    pub fn new() -> Self {
        Self {
            browser_panel_shown: true,
            browser_panel_tab: BrowserPanelTab::Samples,
            browser_panel_width: 200.0,
            browser_panel_search_text: String::new(),
            browser_panel_volume_normalized: 0.75,
            browser_current_directory: "../testtesttest".into(),
            selected_browser_entry: None,

            browser_list_entries: vec![
                BrowserListEntry {
                    type_: BrowserListEntryType::Folder,
                    text: "test_folder_1".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::Folder,
                    text: "test_folder_2".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::Folder,
                    text: "test_folder_3".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::AudioFile,
                    text: "test_file_1.wav".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::AudioFile,
                    text: "test_file_2.wav".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::AudioFile,
                    text: "test_file_3.wav".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::AudioFile,
                    text: "test_file_4.wav".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::AudioFile,
                    text: "test_file_5.wav".into(),
                    selected: false,
                },
                BrowserListEntry {
                    type_: BrowserListEntryType::AudioFile,
                    text: "test_file_6.wav".into(),
                    selected: false,
                },
            ],
        }
    }
}

impl Model for BoundUiState {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {}
}
