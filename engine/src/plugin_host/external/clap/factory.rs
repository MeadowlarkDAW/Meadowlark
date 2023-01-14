use super::*;

use crate::utils::thread_id::SharedThreadIDs;
use basedrop::Shared;
use clack_host::bundle::PluginBundle;
use clack_host::factory::PluginFactory as RawClapPluginFactory;
use clack_host::instance::PluginInstance;
use meadowlark_plugin_api::HostRequestChannelSender;
use meadowlark_plugin_api::{
    HostInfo, PluginDescriptor, PluginFactory, PluginInstanceID, PluginMainThread,
};
use std::ffi::CStr;
use std::path::PathBuf;

pub(crate) struct ClapPluginFactory {
    pub clap_version: clack_host::version::ClapVersion,

    bundle: PluginBundle,
    descriptor: PluginDescriptor,
    id: Shared<String>,

    thread_ids: SharedThreadIDs,
}

impl PluginFactory for ClapPluginFactory {
    fn description(&self) -> PluginDescriptor {
        self.descriptor.clone()
    }

    /// Create a new instance of this plugin.
    ///
    /// **NOTE**: The plugin is **NOT** allowed to use the host callbacks in this method.
    ///
    /// A `basedrop` collector handle is provided for realtime-safe garbage collection.
    ///
    /// `[main-thread]`
    fn instantiate(
        &mut self,
        host_request: HostRequestChannelSender,
        host_info: Shared<HostInfo>,
        plugin_id: PluginInstanceID,
        coll_handle: &basedrop::Handle,
    ) -> Result<Box<dyn PluginMainThread>, String> {
        log::trace!("clap plugin factory new {}", &*self.descriptor.id);

        let id = Shared::clone(plugin_id.rdn());

        // TODO: this is a little wasteful, the PluginHost should be able to fit somewhere else
        let host = host_info.clack_host_info.clone();

        let raw_plugin = match PluginInstance::new(
            |_| {
                ClapHostShared::new(
                    id,
                    host_request,
                    self.thread_ids.clone(),
                    plugin_id,
                    coll_handle,
                )
            },
            |shared| ClapHostMainThread::new(shared),
            &self.bundle,
            self.id.as_bytes(),
            &host,
        ) {
            Ok(plugin) => plugin,
            Err(e) => return Err(format!("{}", e)),
        };

        Ok(Box::new(ClapPluginMainThread::new(raw_plugin)?))
    }
}

pub(crate) fn entry_init(
    plugin_path: &PathBuf,
    thread_ids: SharedThreadIDs,
    coll_handle: &basedrop::Handle,
) -> Result<Vec<ClapPluginFactory>, String> {
    log::trace!("clap entry init at path {:?}", plugin_path);
    let bundle = PluginBundle::load(plugin_path)
        .map_err(|e| format!("Failed to load plugin bundle from path {:?}: {}", plugin_path, e))?;

    let factory = bundle.get_factory::<RawClapPluginFactory>().ok_or_else(|| {
        format!(
            "Plugin from path {:?} returned null while calling clap_plugin_entry.get_factory()",
            plugin_path
        )
    })?;

    let num_plugins = factory.plugin_count();
    if num_plugins == 0 {
        return Err(format!(
            "Plugin from path {:?} returned 0 while calling clap_plugin_factory.get_plugin_count()",
            plugin_path
        ));
    }

    let mut factories: Vec<ClapPluginFactory> = Vec::with_capacity(num_plugins);

    for i in 0..num_plugins {
        // Safe because this is the correct format of this function as described in the
        // CLAP spec.
        let descriptor = factory.plugin_descriptor(i);

        log::trace!("clap plugin instance parse descriptor {:?}", plugin_path);

        let descriptor = match parse_clap_plugin_descriptor(descriptor, plugin_path, i) {
            Ok(descriptor) => descriptor,
            Err(e) => {
                return Err(e);
            }
        };
        log::trace!("clap plugin instance parse descriptor DONE {:?}", plugin_path);

        let id = Shared::new(coll_handle, descriptor.id.clone());

        factories.push(ClapPluginFactory {
            bundle: bundle.clone(),
            descriptor,
            id,
            clap_version: bundle.version(),
            thread_ids: thread_ids.clone(),
        });
    }

    Ok(factories)
}

fn parse_clap_plugin_descriptor(
    raw: Option<clack_host::bundle::PluginDescriptor>,
    plugin_path: &PathBuf,
    plugin_index: usize,
) -> Result<PluginDescriptor, String> {
    let raw = raw.ok_or_else(|| {
        format!(
            "Plugin from path {:?} returned null for its clap_plugin_descriptor at index: {}",
            plugin_path, plugin_index
        )
    })?;

    let parse_optional = |raw_s: Option<&CStr>, field: &'static str| -> String {
        raw_s
            .map(|s| {
                s.to_str().unwrap_or_else(|e| {
                    log::warn!("failed to parse {} from clap_plugin_descriptor: {}", field, e);
                    ""
                })
            })
            .unwrap_or_default()
            .to_string()
    };

    let parse_mandatory = |raw_s: Option<&CStr>, field: &'static str| -> Result<String, String> {
        match parse_optional(raw_s, field) {
            s if !s.is_empty() => Ok(s),
            _ => Err(format!("clap_plugin_descriptor has no {}", field)),
        }
    };

    let id = parse_mandatory(raw.id(), "id")?;

    let version = parse_optional(raw.version(), "version");
    let name = parse_optional(raw.name(), "name");
    let vendor = parse_optional(raw.vendor(), "vendor");
    let description = parse_optional(raw.description(), "description");
    let url = parse_optional(raw.url(), "url");
    let manual_url = parse_optional(raw.manual_url(), "manual_url");
    let support_url = parse_optional(raw.support_url(), "support_url");

    let features: Vec<_> = raw.features().filter_map(|f| f.to_str().ok()).collect();
    let features = features.join(";");

    Ok(PluginDescriptor {
        id,
        name,
        version,
        vendor,
        description,
        url,
        manual_url,
        support_url,
        features,
    })
}
