use clack_host::instance::PluginInstance;

pub(crate) mod factory;

mod host;
use host::*;

mod plugin;

mod process;

use plugin::AudioPortChannels;

pub struct ClapPluginMainThread {
    instance: PluginInstance<ClapHost>,
    audio_port_channels: AudioPortChannels,
}
