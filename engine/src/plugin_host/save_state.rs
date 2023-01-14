use std::fmt::Debug;

use crate::plugin_scanner::ScannedPluginKey;
use clack_extensions::gui::GuiSize;
use meadowlark_plugin_api::ext::audio_ports::PluginAudioPortsExt;
use meadowlark_plugin_api::ext::note_ports::PluginNotePortsExt;

#[derive(Clone)]
pub struct PluginHostSaveState {
    pub key: ScannedPluginKey,

    /// If this is `false` when receiving a save state, then it means that
    /// the plugin was manually deactivated at the time of collecting the
    /// save state of the plugin/project.
    ///
    /// If this is `false` when loading a new plugin, then the plugin will
    /// not be activated automatically.
    pub active: bool,

    /// `True` if this plugin was manually bypassed at the time of collecting
    /// the save state of the plugin/project.
    pub bypassed: bool,

    /// Use this as a backup in case the plugin fails to load. (Most
    /// likey from a user opening another user's project, but the
    /// former user doesn't have this plugin installed on their system.)
    pub backup_audio_ports_ext: Option<PluginAudioPortsExt>,

    /// Use this as a backup in case the plugin fails to load. (Most
    /// likey from a user opening another user's project, but the
    /// former user doesn't have this plugin installed on their system.)
    pub backup_note_ports_ext: Option<PluginNotePortsExt>,

    /// The latest recorded size of the plugin's GUI.
    pub gui_size: Option<GuiSize>,

    /// The plugin's state/preset as raw bytes.
    ///
    /// If this is `None`, then the plugin will load its default
    /// state/preset.
    pub raw_state: Option<Vec<u8>>,
}

impl PluginHostSaveState {
    pub fn new_with_default_state(key: ScannedPluginKey) -> Self {
        Self {
            key,
            active: true,
            bypassed: false,
            backup_audio_ports_ext: None,
            backup_note_ports_ext: None,
            gui_size: None,
            raw_state: None,
        }
    }
}

impl Debug for PluginHostSaveState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("PluginHostSaveState");

        f.field("key", &self.key);
        f.field("active", &self.active);
        f.field("bypassed", &self.bypassed);
        f.field("backup_audio_ports_ext", &self.backup_audio_ports_ext);
        f.field("backup_note_ports_ext", &self.backup_note_ports_ext);
        f.field("gui_size", &self.gui_size);

        if let Some(s) = &self.raw_state {
            f.field("raw_state size", &format!("{}", s.len()));
        } else {
            f.field("raw_state", &"None");
        }

        f.finish()
    }
}
