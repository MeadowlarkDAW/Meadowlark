use audio_graph::NodeID;
use basedrop::Shared;
use fnv::FnvHashMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use walkdir::WalkDir;

use meadowlark_plugin_api::{
    HostInfo, HostRequestChannelReceiver, PluginDescriptor, PluginFactory, PluginFormat,
    PluginInstanceID, PluginInstanceType,
};

use crate::engine::error::NewPluginInstanceError;
use crate::plugin_host::{PluginHostMainThread, PluginHostSaveState};
use crate::utils::thread_id::SharedThreadIDs;

mod missing_plugin;
use missing_plugin::MissingPluginMainThread;

#[cfg(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
const DEFAULT_CLAP_SCAN_DIRECTORIES: [&str; 1] = ["/usr/lib/clap"];

#[cfg(target_os = "macos")]
const DEFAULT_CLAP_SCAN_DIRECTORIES: [&str; 1] = ["/Library/Audio/Plug-Ins/CLAP"];

#[cfg(target_os = "windows")]
// TODO: Find the proper "Common Files" folder at runtime.
const DEFAULT_CLAP_SCAN_DIRECTORIES: [&str; 1] = ["C:/Program Files/Common Files/CLAP"];

const MAX_SCAN_DEPTH: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScannedPluginKey {
    pub rdn: String,
    pub format: PluginFormat,
}

#[derive(Debug, Clone)]
pub struct ScannedPluginInfo {
    pub description: PluginDescriptor,
    pub format: PluginFormat,
    pub format_version: String,
    pub key: ScannedPluginKey,
}

impl ScannedPluginInfo {
    pub fn rdn(&self) -> &str {
        self.key.rdn.as_str()
    }
}

struct LoadedPluginFactory {
    factory: Box<dyn PluginFactory>,
    format: PluginFormat,
    shared_rdn: Shared<String>,
}

struct ScannedPluginBundle {
    binary_path: Option<PathBuf>,

    loaded_factories: Option<HashMap<String, LoadedPluginFactory>>,
}

pub(crate) struct PluginScanner {
    scanned_internal_plugins: HashMap<ScannedPluginKey, ScannedPluginBundle>,

    scanned_external_plugins: HashMap<ScannedPluginKey, u32>,
    external_plugin_bundles: FnvHashMap<u32, ScannedPluginBundle>,

    clap_scan_directories: Vec<PathBuf>,

    host_info: Shared<HostInfo>,

    thread_ids: SharedThreadIDs,

    next_plug_unique_id: u64,

    coll_handle: basedrop::Handle,
}

impl PluginScanner {
    pub fn new(
        coll_handle: basedrop::Handle,
        host_info: Shared<HostInfo>,
        thread_ids: SharedThreadIDs,
    ) -> Self {
        Self {
            scanned_internal_plugins: HashMap::default(),
            scanned_external_plugins: HashMap::default(),
            external_plugin_bundles: FnvHashMap::default(),

            clap_scan_directories: Vec::new(),

            host_info,

            thread_ids,

            // IDs 0 and 1 are used exclusively by the graph_in_node and graph_out_node
            // respectively.
            next_plug_unique_id: 2,

            coll_handle,
        }
    }

    pub fn add_clap_scan_directory(&mut self, path: PathBuf) -> bool {
        // Check if the path is already a default path.
        for p in DEFAULT_CLAP_SCAN_DIRECTORIES.iter() {
            if path == PathBuf::from_str(p).unwrap() {
                log::warn!("Path is already a default scan directory {:?}", &path);
                return false;
            }
        }

        if !self.clap_scan_directories.contains(&path) {
            // Make sure the directory exists.
            match std::fs::read_dir(&path) {
                Ok(_) => {
                    log::info!("Added plugin scan directory {:?}", &path);
                    self.clap_scan_directories.push(path);
                    true
                }
                Err(e) => {
                    log::error!("Failed to add plugin scan directory {:?}: {}", &path, e);
                    false
                }
            }
        } else {
            log::warn!("Already added plugin scan directory {:?}", &path);
            false
        }
    }

    pub fn remove_clap_scan_directory(&mut self, path: PathBuf) -> bool {
        let mut remove_i = None;
        for (i, p) in self.clap_scan_directories.iter().enumerate() {
            if &path == p {
                remove_i = Some(i);
                break;
            }
        }

        if let Some(i) = remove_i {
            self.clap_scan_directories.remove(i);

            log::info!("Removed plugin scan directory {:?}", &path);

            true
        } else {
            log::warn!("Already removed plugin scan directory {:?}", &path);
            false
        }
    }

    pub fn scan_internal_plugin(
        &mut self,
        factory: Box<dyn PluginFactory>,
    ) -> Result<ScannedPluginKey, String> {
        let description = factory.description();

        let key =
            ScannedPluginKey { rdn: description.id.to_string(), format: PluginFormat::Internal };

        if self.scanned_internal_plugins.contains_key(&key) {
            log::warn!("Already scanned internal plugin: {:?}", &key);
        }

        let mut loaded_factories: HashMap<String, LoadedPluginFactory> = HashMap::default();
        loaded_factories.insert(
            description.id.to_string(),
            LoadedPluginFactory {
                factory,
                format: PluginFormat::Internal,
                shared_rdn: Shared::new(&self.coll_handle, description.id),
            },
        );

        let scanned_plugin =
            ScannedPluginBundle { binary_path: None, loaded_factories: Some(loaded_factories) };

        let _ = self.scanned_internal_plugins.insert(key.clone(), scanned_plugin);

        Ok(key)
    }

    pub fn scan_external_plugins(&mut self) -> ScanExternalPluginsRes {
        log::info!("(Re)scanning plugin directories...");

        // TODO: Detect duplicate plugins (both duplicates with different versions and with different formats)

        // TODO: Scan plugins in a separate thread?

        self.scanned_external_plugins.clear();
        self.external_plugin_bundles.clear();
        let mut scanned_plugins: Vec<ScannedPluginInfo> = Vec::new();
        let mut failed_plugins: Vec<(PathBuf, String)> = Vec::new();

        let mut next_external_factory_key: u32 = 0;

        /*
        for (key, f) in self.scanned_internal_plugins.iter() {
            scanned_plugins.push(ScannedPlugin {
                description: f.factory.description(),
                format: PluginFormat::Internal,
                format_version: env!("CARGO_PKG_VERSION").into(),
                key: key.clone(),
            })
        }
        */

        {
            let mut found_binaries: Vec<PathBuf> = Vec::new();

            let mut scan_directories: Vec<PathBuf> = DEFAULT_CLAP_SCAN_DIRECTORIES
                .iter()
                .map(|s| PathBuf::from_str(s).unwrap())
                .collect();

            if let Some(mut dir) = dirs::home_dir() {
                dir.push(".clap");
                scan_directories.push(dir);
            } else {
                log::warn!("Could not search local clap plugin directory: Could not get user's home directory");
            }

            for dir in scan_directories.iter().chain(self.clap_scan_directories.iter()) {
                let walker = WalkDir::new(dir).max_depth(MAX_SCAN_DEPTH).follow_links(true);

                for item in walker {
                    match item {
                        Ok(binary) => {
                            if !binary.file_type().is_file() {
                                continue;
                            }

                            match binary.path().extension().and_then(|e| e.to_str()) {
                                Some(ext) if ext == "clap" => {}
                                _ => continue,
                            };

                            let binary_path = binary.into_path();
                            log::trace!("Found CLAP binary: {:?}", &binary_path);
                            found_binaries.push(binary_path);
                        }
                        Err(e) => {
                            log::warn!("Failed to scan binary for potential CLAP plugin: {}", e);
                        }
                    }
                }
            }

            for binary_path in found_binaries.iter() {
                match crate::plugin_host::external::clap::factory::entry_init(
                    binary_path,
                    self.thread_ids.clone(),
                    &self.coll_handle,
                ) {
                    Ok(mut factories) => {
                        let _ = self.external_plugin_bundles.insert(
                            next_external_factory_key,
                            ScannedPluginBundle {
                                binary_path: Some(binary_path.clone()),
                                // We will reload the factories once a plugin is added to the graph.
                                loaded_factories: None,
                            },
                        );

                        for f in factories.drain(..) {
                            let id: String = f.description().id.clone();
                            let v = f.clap_version;
                            let format_version = format!("{}.{}.{}", v.major, v.minor, v.revision);

                            log::debug!(
                                "Successfully scanned CLAP plugin with ID: {}, version {}, and CLAP version {}",
                                &id,
                                &f.description().version,
                                &format_version,
                            );
                            log::trace!("Full plugin descriptor: {:?}", f.description());

                            let key =
                                ScannedPluginKey { rdn: id.clone(), format: PluginFormat::Clap };

                            let description = f.description();

                            scanned_plugins.push(ScannedPluginInfo {
                                description,
                                format: PluginFormat::Clap,
                                format_version,
                                key: key.clone(),
                            });

                            if self
                                .scanned_external_plugins
                                .insert(key, next_external_factory_key)
                                .is_some()
                            {
                                // TODO: Handle this better
                                log::warn!("Found duplicate CLAP plugins with ID: {}", &id);
                                let _ = scanned_plugins.pop();
                            }
                        }

                        next_external_factory_key += 1;
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to scan CLAP plugin binary at {:?}: {}",
                            binary_path,
                            e
                        );
                        failed_plugins.push((binary_path.clone(), e));
                    }
                }
            }
        }

        ScanExternalPluginsRes { scanned_plugins, failed_plugins }
    }

    pub(crate) fn create_plugin(
        &mut self,
        mut save_state: PluginHostSaveState,
        node_id: NodeID,
        fallback_to_other_formats: bool,
    ) -> CreatePluginResult {
        // TODO: return an actual result
        let mut plugin_bundle = None;
        let mut status = Ok(());
        let mut loaded = false;

        // Always try to use internal plugins when available.
        if save_state.key.format == PluginFormat::Internal || fallback_to_other_formats {
            let pb = if save_state.key.format == PluginFormat::Internal {
                self.scanned_internal_plugins.get_mut(&save_state.key)
            } else {
                let new_key = ScannedPluginKey {
                    rdn: save_state.key.rdn.clone(),
                    format: PluginFormat::Internal,
                };
                self.scanned_internal_plugins.get_mut(&new_key)
            };

            if let Some(pb) = pb {
                plugin_bundle = Some(pb);
            } else {
                status = Err(NewPluginInstanceError::FormatNotFound(
                    save_state.key.rdn.clone(),
                    PluginFormat::Internal,
                ));
            }
        }

        // Next try to use the clap version of the plugin.
        if plugin_bundle.is_none()
            && (save_state.key.format == PluginFormat::Clap || fallback_to_other_formats)
        {
            let pb = if save_state.key.format == PluginFormat::Clap {
                self.scanned_external_plugins.get(&save_state.key)
            } else {
                let new_key = ScannedPluginKey {
                    rdn: save_state.key.rdn.clone(),
                    format: PluginFormat::Clap,
                };
                self.scanned_external_plugins.get(&new_key)
            };

            if let Some(plugin_bundle_key) = &pb {
                plugin_bundle = self.external_plugin_bundles.get_mut(plugin_bundle_key);
            } else {
                status = Err(NewPluginInstanceError::FormatNotFound(
                    save_state.key.rdn.clone(),
                    PluginFormat::Clap,
                ));
            }
        }

        let mut format = PluginInstanceType::Unloaded;

        let (host_request_rx, channel_send) =
            HostRequestChannelReceiver::new_channel(self.thread_ids.main_thread_id().unwrap());

        let plugin_factory = if let Some(plugin_bundle) = plugin_bundle {
            let loaded_factories =
                if let Some(loaded_factories) = plugin_bundle.loaded_factories.as_mut() {
                    Some(loaded_factories)
                } else {
                    // Reload the plugin factories from disk.
                    match crate::plugin_host::external::clap::factory::entry_init(
                        plugin_bundle.binary_path.as_ref().unwrap(),
                        self.thread_ids.clone(),
                        &self.coll_handle,
                    ) {
                        Ok(mut factories) => {
                            let mut loaded_factories: HashMap<String, LoadedPluginFactory> =
                                HashMap::default();
                            for f in factories.drain(..) {
                                let rdn = f.description().id.clone();

                                let _ = loaded_factories.insert(
                                    f.description().id.clone(),
                                    LoadedPluginFactory {
                                        factory: Box::new(f),
                                        format: PluginFormat::Clap,
                                        shared_rdn: Shared::new(&self.coll_handle, rdn),
                                    },
                                );
                            }

                            plugin_bundle.loaded_factories = Some(loaded_factories);

                            plugin_bundle.loaded_factories.as_mut()
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to load CLAP binary at path {:?}: {}",
                                plugin_bundle.binary_path.as_ref().unwrap(),
                                e
                            );
                            None
                        }
                    }
                };

            if let Some(loaded_factories) = loaded_factories {
                match loaded_factories.get_mut(&save_state.key.rdn) {
                    Some(f) => Some(f),
                    None => {
                        log::error!(
                            "Failed to find plugin with ID {} in CLAP binary at path {:?}",
                            &save_state.key.rdn,
                            plugin_bundle.binary_path.as_ref().unwrap()
                        );
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let plugin_host = if let Some(plugin_factory) = plugin_factory {
            format = plugin_factory.format.into();

            if save_state.key.format != plugin_factory.format {
                save_state.key = ScannedPluginKey {
                    rdn: save_state.key.rdn.clone(),
                    format: plugin_factory.format,
                };
            }

            let id = PluginInstanceID::_new(
                node_id.into(),
                self.next_plug_unique_id,
                format,
                Shared::clone(&plugin_factory.shared_rdn),
            );
            self.next_plug_unique_id += 1;

            let plug_main_thread = match plugin_factory.factory.instantiate(
                channel_send,
                self.host_info.clone(),
                id.clone(),
                &self.coll_handle,
            ) {
                Ok(plug_main_thread) => {
                    status = Ok(());
                    loaded = true;

                    plug_main_thread
                }
                Err(e) => {
                    status = Err(NewPluginInstanceError::FactoryFailedToCreateNewInstance(
                        (*plugin_factory.shared_rdn).clone(),
                        e,
                    ));

                    Box::new(MissingPluginMainThread::new(
                        save_state.key.clone(),
                        save_state.backup_audio_ports_ext.clone(),
                        save_state.backup_note_ports_ext.clone(),
                    ))
                }
            };

            PluginHostMainThread::new(
                id,
                save_state,
                plug_main_thread,
                host_request_rx,
                loaded,
                &self.coll_handle,
            )
        } else {
            let rdn = Shared::new(&self.coll_handle, save_state.key.rdn.clone());

            let id = PluginInstanceID::_new(node_id.into(), self.next_plug_unique_id, format, rdn);
            self.next_plug_unique_id += 1;

            if status.is_ok() {
                status = Err(NewPluginInstanceError::NotFound(save_state.key.rdn.clone()));
            }

            let plug_main_thread = Box::new(MissingPluginMainThread::new(
                save_state.key.clone(),
                save_state.backup_audio_ports_ext.clone(),
                save_state.backup_note_ports_ext.clone(),
            ));

            PluginHostMainThread::new(
                id,
                save_state,
                plug_main_thread,
                host_request_rx,
                loaded,
                &self.coll_handle,
            )
        };

        CreatePluginResult { plugin_host, status }
    }

    pub(crate) fn unload_unused_binaries(&mut self) {
        // TODO: Unload all external plugin binaries that are no longer being
        // used. (Perhaps by counting how many references are left in the `Shared`
        // pointers?)
    }
}

pub(crate) struct CreatePluginResult {
    pub plugin_host: PluginHostMainThread,
    pub status: Result<(), NewPluginInstanceError>,
}

#[derive(Debug)]
pub struct ScanExternalPluginsRes {
    pub scanned_plugins: Vec<ScannedPluginInfo>,
    pub failed_plugins: Vec<(PathBuf, String)>,
}
