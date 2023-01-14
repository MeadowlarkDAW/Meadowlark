use meadowlark_plugin_api::ext::audio_ports::PluginAudioPortsExt;
use meadowlark_plugin_api::ext::note_ports::PluginNotePortsExt;
use meadowlark_plugin_api::{PluginActivatedInfo, PluginMainThread};

use crate::plugin_scanner::ScannedPluginKey;

pub(super) struct MissingPluginMainThread {
    key: ScannedPluginKey,
    backup_audio_ports_ext: Option<PluginAudioPortsExt>,
    backup_note_ports_ext: Option<PluginNotePortsExt>,
}

impl MissingPluginMainThread {
    pub fn new(
        key: ScannedPluginKey,
        backup_audio_ports_ext: Option<PluginAudioPortsExt>,
        backup_note_ports_ext: Option<PluginNotePortsExt>,
    ) -> Self {
        Self { key, backup_audio_ports_ext, backup_note_ports_ext }
    }
}

impl PluginMainThread for MissingPluginMainThread {
    fn activate(
        &mut self,
        _sample_rate: u32,
        _min_frames: u32,
        _max_frames: u32,
        _coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String> {
        Err(format!("Plugin with key {:?} is missing on the system.", &self.key))
    }

    fn collect_save_state(&mut self) -> Result<Option<Vec<u8>>, String> {
        Err(format!("Plugin with key {:?} is missing on the system.", &self.key))
    }

    fn load_save_state(&mut self, _state: Vec<u8>) -> Result<(), String> {
        Err(format!("Plugin with key {:?} is missing on the system.", &self.key))
    }

    fn audio_ports_ext(&mut self) -> Result<PluginAudioPortsExt, String> {
        if let Some(a) = &self.backup_audio_ports_ext {
            Ok(a.clone())
        } else {
            Err(format!("Plugin with key {:?} is missing on the system.", &self.key))
        }
    }

    fn note_ports_ext(&mut self) -> Result<PluginNotePortsExt, String> {
        if let Some(n) = &self.backup_note_ports_ext {
            Ok(n.clone())
        } else {
            Err(format!("Plugin with key {:?} is missing on the system.", &self.key))
        }
    }
}
