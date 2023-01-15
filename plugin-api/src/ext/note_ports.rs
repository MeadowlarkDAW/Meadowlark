//! This extension provides a way for the plugin to describe its current note ports.
//! If the plugin does not implement this extension, it won't have note input or output.
//! The plugin is only allowed to change its note ports configuration while it is deactivated.

use clack_extensions::note_ports::{NoteDialect, NoteDialects};

pub(crate) static EMPTY_NOTE_PORTS_CONFIG: PluginNotePortsExt = PluginNotePortsExt::empty();

#[derive(Debug, Clone, PartialEq, Eq)]
/// The layout of the audio ports of a plugin.
pub struct PluginNotePortsExt {
    /// The list of input note ports.
    pub inputs: Vec<NotePortInfo>,

    /// The list of output note ports.
    pub outputs: Vec<NotePortInfo>,
}

impl PluginNotePortsExt {
    pub const fn empty() -> Self {
        Self { inputs: Vec::new(), outputs: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotePortInfo {
    /// stable identifier
    pub stable_id: u32,

    /// bitfield, see `NoteDialect`
    pub supported_dialects: NoteDialects,

    /// one value of `NoteDialect`
    pub preferred_dialect: Option<NoteDialect>,

    /// displayable name
    pub display_name: Option<String>,
}
