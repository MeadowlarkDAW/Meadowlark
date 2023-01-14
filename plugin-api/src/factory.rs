use basedrop::Shared;

use super::{
    HostInfo, HostRequestChannelSender, PluginDescriptor, PluginInstanceID, PluginMainThread,
};

/// The methods of an audio plugin which are used to create new instances of the plugin.
pub trait PluginFactory: Send {
    fn description(&self) -> PluginDescriptor;

    /// Create a new instance of this plugin.
    ///
    /// A `basedrop` collector handle is provided for realtime-safe garbage collection.
    ///
    /// `[main-thread]`
    fn instantiate(
        &mut self,
        host_request_channel: HostRequestChannelSender,
        host_info: Shared<HostInfo>,
        plugin_id: PluginInstanceID,
        coll_handle: &basedrop::Handle,
    ) -> Result<Box<dyn PluginMainThread>, String>;
}
