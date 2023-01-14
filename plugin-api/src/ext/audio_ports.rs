pub const PORT_TYPE_MONO: &str = "mono";
pub const PORT_TYPE_STEREO: &str = "stereo";

pub const PORT_NAME_SIDECHAIN: &str = "sidechain";

pub(crate) static EMPTY_AUDIO_PORTS_CONFIG: PluginAudioPortsExt = PluginAudioPortsExt::empty();

/// The layout of the audio ports of a plugin.
///
/// By default this returns a configuration with a main stereo
/// input port and a main stereo output port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginAudioPortsExt {
    /// The list of input audio ports, in order.
    pub inputs: Vec<AudioPortInfo>,

    /// The list of output audio ports, in order.
    pub outputs: Vec<AudioPortInfo>,

    /// Specifies which audio ports are "main" ports.
    pub main_ports_layout: MainPortsLayout,
}

impl Default for PluginAudioPortsExt {
    fn default() -> Self {
        PluginAudioPortsExt::stereo_in_out()
    }
}

impl PluginAudioPortsExt {
    pub fn total_in_channels(&self) -> usize {
        let mut num_inputs = 0;
        for input in self.inputs.iter() {
            num_inputs += input.channels;
        }
        num_inputs as usize
    }

    pub fn total_out_channels(&self) -> usize {
        let mut num_outputs = 0;
        for output in self.outputs.iter() {
            num_outputs += output.channels;
        }
        num_outputs as usize
    }

    pub fn in_channel_index(&self, port_stable_id: u32, port_channel: u16) -> Option<usize> {
        // TODO: Optimize this? This should be cached in PortPortChannelID

        let mut channel_i: u16 = 0;

        for p in self.inputs.iter() {
            if p.stable_id == port_stable_id {
                return if port_channel < p.channels {
                    Some((channel_i + port_channel) as usize)
                } else {
                    None
                };
            } else {
                channel_i += p.channels;
            }
        }

        None
    }

    pub fn out_channel_index(&self, port_stable_id: u32, port_channel: u16) -> Option<usize> {
        // TODO: Optimize this? This should be cached in PortPortChannelID

        let mut channel_i: u16 = 0;

        for p in self.outputs.iter() {
            if p.stable_id == port_stable_id {
                return if port_channel < p.channels {
                    Some((channel_i + port_channel) as usize)
                } else {
                    None
                };
            } else {
                channel_i += p.channels;
            }
        }

        None
    }

    /// Returns the input port with the largest number of channels.
    pub fn max_input_channels(&self) -> Option<&AudioPortInfo> {
        if self.inputs.is_empty() {
            return None;
        }

        let mut max = &self.inputs[0];
        for p in self.inputs.iter() {
            if p.channels > max.channels {
                max = p;
            }
        }

        Some(max)
    }

    /// Returns the output port with the largest number of channels.
    pub fn max_output_channels(&self) -> Option<&AudioPortInfo> {
        if self.outputs.is_empty() {
            return None;
        }

        let mut max = &self.outputs[0];
        for p in self.outputs.iter() {
            if p.channels > max.channels {
                max = p;
            }
        }

        Some(max)
    }

    pub const fn empty() -> Self {
        PluginAudioPortsExt {
            inputs: vec![],
            outputs: vec![],
            main_ports_layout: MainPortsLayout::NoMainPorts,
        }
    }

    /// A main stereo input port and a main stereo output port.
    pub fn stereo_in_out() -> Self {
        PluginAudioPortsExt {
            inputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: None,
            }],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::InOut,
        }
    }

    /// A main mono input port and a main mono output port.
    pub fn mono_in_out() -> Self {
        PluginAudioPortsExt {
            inputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: None,
            }],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::InOut,
        }
    }

    /// A main stereo output port only.
    pub fn stereo_out() -> Self {
        PluginAudioPortsExt {
            inputs: vec![],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::OutOnly,
        }
    }

    /// A main mono output port only.
    pub fn mono_out() -> Self {
        PluginAudioPortsExt {
            inputs: vec![],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::OutOnly,
        }
    }

    /// A main stereo input port and a main stereo output port, with an
    /// additional stereo sidechain input.
    pub fn stereo_in_out_w_sidechain() -> Self {
        PluginAudioPortsExt {
            inputs: vec![
                AudioPortInfo {
                    stable_id: 0,
                    channels: 2,
                    port_type: Some(PORT_TYPE_STEREO.into()),
                    display_name: None,
                },
                AudioPortInfo {
                    stable_id: 1,
                    channels: 2,
                    port_type: Some(PORT_TYPE_STEREO.into()),
                    display_name: Some(PORT_NAME_SIDECHAIN.into()),
                },
            ],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::InOut,
        }
    }

    /// A main mono input port and a main mono output port, with an
    /// additional mono sidechain input.
    pub fn mono_in_out_w_sidechain() -> Self {
        PluginAudioPortsExt {
            inputs: vec![
                AudioPortInfo {
                    stable_id: 0,
                    channels: 1,
                    port_type: Some(PORT_TYPE_MONO.into()),
                    display_name: None,
                },
                AudioPortInfo {
                    stable_id: 1,
                    channels: 1,
                    port_type: Some(PORT_TYPE_MONO.into()),
                    display_name: Some(PORT_NAME_SIDECHAIN.into()),
                },
            ],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::InOut,
        }
    }

    /// A main stereo output port with an additional stereo sidechain
    /// input.
    pub fn stereo_out_w_sidechain() -> Self {
        PluginAudioPortsExt {
            inputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: Some(PORT_NAME_SIDECHAIN.into()),
            }],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::OutOnly,
        }
    }

    /// A main mono output port with an additional mono sidechain
    /// input.
    pub fn mono_out_w_sidechain() -> Self {
        PluginAudioPortsExt {
            inputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: Some(PORT_NAME_SIDECHAIN.into()),
            }],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::OutOnly,
        }
    }

    /// A main mono input port and a main stereo output port.
    pub fn mono_in_stereo_out() -> Self {
        PluginAudioPortsExt {
            inputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: None,
            }],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::InOut,
        }
    }

    /// A main stereo input port and a main mono output port.
    pub fn stereo_in_mono_out() -> Self {
        PluginAudioPortsExt {
            inputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 2,
                port_type: Some(PORT_TYPE_STEREO.into()),
                display_name: None,
            }],
            outputs: vec![AudioPortInfo {
                stable_id: 0,
                channels: 1,
                port_type: Some(PORT_TYPE_MONO.into()),
                display_name: None,
            }],
            main_ports_layout: MainPortsLayout::InOut,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Information about a custom audio port.
pub struct AudioPortInfo {
    /// Stable identifier, it must never change.
    pub stable_id: u32,

    /// The number of channels in this port.
    ///
    /// This cannot be `0`.
    pub channels: u16,

    /// If `None` or empty then it is unspecified (arbitrary audio).
    ///
    /// This can be compared against:
    /// - `PORT_TYPE_MONO`  ("mono")
    /// - `PORT_TYPE_STEREO`  ("stereo")
    /// - `PORT_TYPE_SURROUND` (defined in the surround extension)
    /// - `PORT_TYPE_AMBISONIC` (defined in the ambisonic extension)
    /// - `PORT_TYPE_CV` (defined in the cv extension)
    ///
    /// An extension can provide its own port type and way to inspect the channels.
    pub port_type: Option<String>,

    /// The displayable name.
    ///
    /// Set this to `None` to use the default name.
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Specifies which audio ports are "main" ports.
pub enum MainPortsLayout {
    /// Both the first input port and the first output port are main ports.
    InOut,

    /// The first input port is a main port, and there are no main output ports.
    InOnly,

    /// The first output port is a main port, and there are no main input ports.
    OutOnly,

    /// There are no main input or output ports.
    NoMainPorts,
}

impl Default for MainPortsLayout {
    fn default() -> Self {
        MainPortsLayout::InOut
    }
}
