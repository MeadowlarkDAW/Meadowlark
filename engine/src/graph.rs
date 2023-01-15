use std::hash::Hash;

use audio_graph::{error::AddEdgeError, AudioGraphHelper, EdgeID, PortID, TypeIdx};
use basedrop::Shared;
use fnv::{FnvHashMap, FnvHashSet};
use meadowlark_plugin_api::transport::LoopState;
use smallvec::SmallVec;

mod compiler;

pub mod error;

pub(crate) mod shared_pools;

use meadowlark_plugin_api::{PluginInstanceID, PluginInstanceType};

use crate::engine::modify_request::{ConnectEdgeReq, EdgeReqPortID};
use crate::engine::timer_wheel::EngineTimerWheel;
use crate::engine::{EngineTempoMap, NewPluginRes, OnIdleEvent, PluginStatus};
use crate::plugin_host::{
    OnIdleResult, PluginHostMainThread, PluginHostProcessorWrapper, PluginHostSaveState,
};
use crate::plugin_scanner::PluginScanner;
use crate::processor_schedule::tasks::{TransportHandle, TransportTask};
use crate::processor_schedule::ProcessorSchedule;
use crate::utils::thread_id::SharedThreadIDs;

use compiler::verifier::Verifier;
use shared_pools::{GraphSharedPools, SharedProcessorSchedule};

use error::{ConnectEdgeError, ConnectEdgeErrorType, GraphCompilerError};

/// A default port type for general purpose applications
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortType {
    /// Audio ports
    Audio = 0,
    Note = 1,
    Automation = 2,
}

impl PortType {
    pub const NUM_TYPES: usize = 3;

    pub const AUDIO_TYPE_IDX: TypeIdx = TypeIdx(PortType::Audio as u32 as usize);
    pub const NOTE_TYPE_IDX: TypeIdx = TypeIdx(PortType::Note as u32 as usize);
    pub const AUTOMATION_TYPE_IDX: TypeIdx = TypeIdx(PortType::Automation as u32 as usize);

    pub const AUDIO_IDX: usize = 0;
    pub const NOTE_IDX: usize = 1;
    pub const AUTOMATION_IDX: usize = 2;

    pub fn from_type_idx(p: TypeIdx) -> Option<Self> {
        match p.0 {
            0 => Some(PortType::Audio),
            1 => Some(PortType::Note),
            2 => Some(PortType::Automation),
            _ => None,
        }
    }

    pub fn as_type_idx(&self) -> TypeIdx {
        TypeIdx(*self as u32 as usize)
    }
}

impl Default for PortType {
    fn default() -> Self {
        PortType::Audio
    }
}

/// The ID for a particular audio, note, or automation port in the
/// audio graph.
///
/// If this is an audio port, then this is the ID of a particular
/// channel on that audio port. Otherwise this is just the ID
/// for the note/automation port altogether.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortChannelID {
    /// The unique (and stable) identifier the plugin has assigned
    /// to this port.
    ///
    /// By "stable" I mean that the ID for this port does not change
    /// between versions of the plugin.
    pub(crate) stable_id: u32,
    pub(crate) port_type: PortType,

    /// `true` if this is an input port, `false` otherwise.
    pub(crate) is_input: bool,

    /// The channel on the audio port. This is irrelevant if this is
    /// not an audio port.
    pub(crate) channel: u16,
}

pub(crate) struct AudioGraph {
    shared_pools: GraphSharedPools,
    verifier: Verifier,

    graph_helper: AudioGraphHelper,
    coll_handle: basedrop::Handle,

    graph_in_id: PluginInstanceID,
    graph_out_id: PluginInstanceID,
    graph_in_num_audio_channels: usize,
    graph_out_num_audio_channels: usize,

    edge_id_to_ds_edge_id: FnvHashMap<EdgeID, EngineEdgeID>,
    next_ds_edge_id: u64,

    sample_rate: u32,
    min_frames: u32,
    max_frames: u32,

    /// For the plugins that are queued to be removed, make sure that
    /// the plugin's processor part is dropped in the process thread.
    plugin_processors_to_drop: Vec<Shared<PluginHostProcessorWrapper>>,

    thread_ids: SharedThreadIDs,

    schedule_version: u64,
}

impl AudioGraph {
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub fn new(
        coll_handle: basedrop::Handle,
        graph_in_channels: usize,
        graph_out_channels: usize,
        sample_rate: u32,
        min_frames: u32,
        max_frames: u32,
        note_buffer_size: usize,
        event_buffer_size: usize,
        thread_ids: SharedThreadIDs,
        seek_to_frame: u64,
        loop_state: LoopState,
        tempo_map: Box<dyn EngineTempoMap>,
        transport_declick_seconds: f64,
        engine_timer: &mut EngineTimerWheel,
    ) -> (Self, SharedProcessorSchedule, TransportHandle) {
        //assert!(graph_in_channels > 0);
        assert!(graph_out_channels > 0);

        let graph_helper = AudioGraphHelper::new(PortType::NUM_TYPES);

        let (transport_task, transport_handle) = TransportTask::new(
            seek_to_frame,
            loop_state,
            sample_rate,
            tempo_map,
            max_frames as usize,
            transport_declick_seconds,
            coll_handle.clone(),
        );

        let (shared_pools, shared_schedule) = GraphSharedPools::new(
            thread_ids.clone(),
            max_frames as usize,
            note_buffer_size,
            event_buffer_size,
            transport_task,
            0,
            coll_handle.clone(),
        );

        let graph_in_rdn = Shared::new(&coll_handle, String::from("app.meadowlark.graph_in_node"));
        let graph_out_rdn =
            Shared::new(&coll_handle, String::from("app.meadowlark.graph_out_node"));

        let graph_in_id =
            PluginInstanceID::_new(0, 0, PluginInstanceType::GraphInput, graph_in_rdn);
        let graph_out_id =
            PluginInstanceID::_new(1, 1, PluginInstanceType::GraphOutput, graph_out_rdn);

        let mut new_self = Self {
            shared_pools,
            verifier: Verifier::new(),
            graph_helper,
            coll_handle,
            graph_in_num_audio_channels: graph_in_channels,
            graph_out_num_audio_channels: graph_out_channels,
            graph_in_id,
            graph_out_id,
            edge_id_to_ds_edge_id: FnvHashMap::default(),
            next_ds_edge_id: 0,
            sample_rate,
            min_frames,
            max_frames,
            plugin_processors_to_drop: Vec::new(),
            thread_ids,
            schedule_version: 0,
        };

        new_self.reset(engine_timer);

        (new_self, shared_schedule, transport_handle)
    }

    pub fn add_new_plugin_instance(
        &mut self,
        save_state: PluginHostSaveState,
        plugin_scanner: &mut PluginScanner,
        fallback_to_other_formats: bool,
    ) -> NewPluginRes {
        let do_activate_plugin = save_state.active;

        let node_id = self.graph_helper.add_node(0.0);
        let res = plugin_scanner.create_plugin(save_state, node_id, fallback_to_other_formats);
        let plugin_id = res.plugin_host.id().clone();

        match res.status {
            Ok(()) => {
                log::debug!("Loaded plugin {:?} successfully", &res.plugin_host.id());
            }
            Err(e) => {
                log::error!(
                    "Failed to load plugin {:?} from save state: {}",
                    &res.plugin_host.id(),
                    e
                );
            }
        }

        let supports_floating_gui = res.plugin_host.supports_floating_gui();
        let supports_embedded_gui = res.plugin_host.supports_embedded_gui();

        if self.shared_pools.plugin_hosts.insert(plugin_id.clone(), res.plugin_host).is_some() {
            panic!("Something went wrong when allocating a new slot for a plugin");
        }

        let activation_status = if do_activate_plugin {
            self.activate_plugin_instance(&plugin_id).unwrap()
        } else {
            PluginStatus::Inactive
        };

        NewPluginRes {
            plugin_id,
            status: activation_status,
            supports_floating_gui,
            supports_embedded_gui,
        }
    }

    pub fn activate_plugin_instance(&mut self, id: &PluginInstanceID) -> Result<PluginStatus, ()> {
        let plugin_host = self.shared_pools.plugin_hosts.get_mut(id).ok_or(())?;

        if let Err(e) = plugin_host.can_activate() {
            return Ok(PluginStatus::ActivationError(e));
        }

        let activation_status = match plugin_host.activate(
            self.sample_rate,
            self.min_frames,
            self.max_frames,
            &mut self.graph_helper,
            &mut self.edge_id_to_ds_edge_id,
            self.thread_ids.clone(),
            self.schedule_version,
            &self.coll_handle,
        ) {
            Ok(res) => PluginStatus::Activated(res),
            Err(e) => PluginStatus::ActivationError(e),
        };

        Ok(activation_status)
    }

    /// Remove the given plugins from the graph.
    ///
    /// This will also automatically disconnect all edges that were connected to these
    /// plugins.
    ///
    /// Requests to remove the "graph input/output" nodes with the IDs
    /// `AudioGraph::graph_in_node_id()` and `AudioGraph::graph_out_node_id()` will be
    /// ignored.
    ///
    /// This returns a list of all the plugins that were successfully removed, as well
    /// as a list of all edges that were removed as a result of the operation.
    pub fn remove_plugin_instances(
        &mut self,
        plugin_ids: &[PluginInstanceID],
        engine_timer: &mut EngineTimerWheel,
    ) -> (FnvHashSet<PluginInstanceID>, Vec<EngineEdgeID>) {
        let mut removed_plugins: FnvHashSet<PluginInstanceID> = FnvHashSet::default();
        let mut removed_edges: Vec<EngineEdgeID> = Vec::new();

        for id in plugin_ids.iter() {
            if id == &self.graph_in_id || id == &self.graph_out_id {
                log::warn!("Ignored request to remove graph in/out node");
                continue;
            }

            if removed_plugins.insert(id.clone()) {
                if let Some(plugin_host) = self.shared_pools.plugin_hosts.get_mut(id) {
                    if let Some(plugin_proc_to_drop) =
                        plugin_host.schedule_remove(&self.coll_handle, engine_timer)
                    {
                        self.plugin_processors_to_drop.push(plugin_proc_to_drop);
                    }

                    let removed_edges_res =
                        self.graph_helper.remove_node(id._node_id().into()).unwrap();
                    for edge_id in removed_edges_res.iter() {
                        if let Some(ds_edge_id) = self.edge_id_to_ds_edge_id.remove(edge_id) {
                            removed_edges.push(ds_edge_id);
                        } else {
                            panic!(
                                "Helper disconnected an edge that doesn't exist in graph: {:?}",
                                edge_id
                            );
                        }
                    }
                } else {
                    removed_plugins.remove(id);
                    log::warn!(
                        "Ignored request to remove plugin instance {:?}: plugin is already removed",
                        id
                    );
                }
            } else {
                log::warn!("Ignored duplicate request to remove plugin instance {:?}", id);
            }
        }

        (removed_plugins, removed_edges)
    }

    pub fn connect_edge(
        &mut self,
        edge: &ConnectEdgeReq,
        src_plugin_id: &PluginInstanceID,
        dst_plugin_id: &PluginInstanceID,
    ) -> Result<Edge, ConnectEdgeError> {
        let (src_port_id, src_port_stable_id) = if src_plugin_id == &self.graph_in_id {
            match &edge.src_port_id {
                EdgeReqPortID::Main => match edge.edge_type {
                    PortType::Audio => {
                        if usize::from(edge.src_port_channel) < self.graph_in_num_audio_channels {
                            (PortID(edge.src_port_channel as u32), edge.src_port_channel as u32)
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                    PortType::Automation => {
                        return Err(ConnectEdgeError {
                            error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                            edge: edge.clone(),
                        });
                    }
                    PortType::Note => {
                        // TODO: Note in/out ports on graph in/out nodes.
                        return Err(ConnectEdgeError {
                            error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                            edge: edge.clone(),
                        });
                    }
                },
                EdgeReqPortID::StableID(_id) => {
                    // TODO: Stable IDs for ports on graph in/out nodes?
                    return Err(ConnectEdgeError {
                        error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                        edge: edge.clone(),
                    });
                }
            }
        } else if let Some(plugin_host) = self.shared_pools.plugin_hosts.get(src_plugin_id) {
            match &edge.src_port_id {
                EdgeReqPortID::Main => match edge.edge_type {
                    PortType::Audio => {
                        if let Some(port_id) = plugin_host
                            .port_ids()
                            .main_audio_out_port_ids
                            .get(usize::from(edge.src_port_channel))
                        {
                            (
                                *port_id,
                                plugin_host
                                    .port_ids()
                                    .port_id_to_channel_id
                                    .get(port_id)
                                    .unwrap()
                                    .stable_id,
                            )
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                    PortType::Note => {
                        if let Some(port_id) = plugin_host.port_ids().main_note_out_port_id {
                            (
                                port_id,
                                plugin_host
                                    .port_ids()
                                    .port_id_to_channel_id
                                    .get(&port_id)
                                    .unwrap()
                                    .stable_id,
                            )
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                    PortType::Automation => {
                        if let Some(port_id) = plugin_host.port_ids().automation_out_port_id {
                            (port_id, 0) // Automation ports always have a stable ID of 0
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                },
                EdgeReqPortID::StableID(id) => {
                    let src_channel_id = PortChannelID {
                        port_type: edge.edge_type,
                        stable_id: *id,
                        is_input: false,
                        channel: edge.src_port_channel,
                    };

                    if let Some(port_id) =
                        plugin_host.port_ids().channel_id_to_port_id.get(&src_channel_id)
                    {
                        (*port_id, *id)
                    } else {
                        return Err(ConnectEdgeError {
                            error_type: ConnectEdgeErrorType::SrcPortDoesNotExist,
                            edge: edge.clone(),
                        });
                    }
                }
            }
        } else {
            return Err(ConnectEdgeError {
                error_type: ConnectEdgeErrorType::SrcPluginDoesNotExist,
                edge: edge.clone(),
            });
        };

        let (dst_port_id, dst_port_stable_id) = if dst_plugin_id == &self.graph_out_id {
            match &edge.dst_port_id {
                EdgeReqPortID::Main => match edge.edge_type {
                    PortType::Audio => {
                        if usize::from(edge.dst_port_channel) < self.graph_out_num_audio_channels {
                            (PortID(edge.dst_port_channel as u32), edge.dst_port_channel as u32)
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                    PortType::Note => {
                        // TODO: Note in/out ports on graph in/out nodes.
                        return Err(ConnectEdgeError {
                            error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                            edge: edge.clone(),
                        });
                    }
                    PortType::Automation => {
                        return Err(ConnectEdgeError {
                            error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                            edge: edge.clone(),
                        });
                    }
                },
                EdgeReqPortID::StableID(_id) => {
                    // TODO: Stable IDs for ports on graph in/out nodes?
                    return Err(ConnectEdgeError {
                        error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                        edge: edge.clone(),
                    });
                }
            }
        } else if let Some(plugin_host) = self.shared_pools.plugin_hosts.get(dst_plugin_id) {
            match &edge.dst_port_id {
                EdgeReqPortID::Main => match edge.edge_type {
                    PortType::Audio => {
                        if let Some(port_id) = plugin_host
                            .port_ids()
                            .main_audio_in_port_ids
                            .get(usize::from(edge.dst_port_channel))
                        {
                            (
                                *port_id,
                                plugin_host
                                    .port_ids()
                                    .port_id_to_channel_id
                                    .get(port_id)
                                    .unwrap()
                                    .stable_id,
                            )
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                    PortType::Note => {
                        if let Some(port_id) = plugin_host.port_ids().main_note_in_port_id {
                            (
                                port_id,
                                plugin_host
                                    .port_ids()
                                    .port_id_to_channel_id
                                    .get(&port_id)
                                    .unwrap()
                                    .stable_id,
                            )
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                    PortType::Automation => {
                        if let Some(port_id) = plugin_host.port_ids().automation_in_port_id {
                            (port_id, 0) // Automation ports always have a stable ID of 0
                        } else {
                            return Err(ConnectEdgeError {
                                error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                                edge: edge.clone(),
                            });
                        }
                    }
                },
                EdgeReqPortID::StableID(id) => {
                    let dst_channel_id = PortChannelID {
                        port_type: edge.edge_type,
                        stable_id: *id,
                        is_input: true,
                        channel: edge.dst_port_channel,
                    };

                    if let Some(port_id) =
                        plugin_host.port_ids().channel_id_to_port_id.get(&dst_channel_id)
                    {
                        (*port_id, *id)
                    } else {
                        return Err(ConnectEdgeError {
                            error_type: ConnectEdgeErrorType::DstPortDoesNotExist,
                            edge: edge.clone(),
                        });
                    }
                }
            }
        } else {
            return Err(ConnectEdgeError {
                error_type: ConnectEdgeErrorType::DstPluginDoesNotExist,
                edge: edge.clone(),
            });
        };

        match self.graph_helper.add_edge(
            src_plugin_id._node_id().into(),
            src_port_id,
            dst_plugin_id._node_id().into(),
            dst_port_id,
            edge.check_for_cycles,
        ) {
            Ok(edge_id) => {
                let ds_edge_id = EngineEdgeID { unique_id: self.next_ds_edge_id, edge_id };
                self.next_ds_edge_id += 1;

                if self.edge_id_to_ds_edge_id.insert(edge_id, ds_edge_id).is_some() {
                    panic!("Something went wrong while connecting edge {:?}", edge_id);
                }

                Ok(Edge {
                    id: ds_edge_id,

                    edge_type: edge.edge_type,

                    src_plugin_id: src_plugin_id.clone(),
                    dst_plugin_id: dst_plugin_id.clone(),

                    src_port_id: src_port_stable_id,
                    dst_port_id: dst_port_stable_id,
                })
            }
            Err(AddEdgeError::CycleDetected) => Err(ConnectEdgeError {
                error_type: ConnectEdgeErrorType::Cycle,
                edge: edge.clone(),
            }),
            Err(AddEdgeError::EdgeAlreadyExists(_)) => Err(ConnectEdgeError {
                error_type: ConnectEdgeErrorType::EdgeAlreadyExists,
                edge: edge.clone(),
            }),
            Err(e) => {
                log::error!("Unexpected error while connecting edge: {}", e);

                Err(ConnectEdgeError {
                    error_type: ConnectEdgeErrorType::Unkown,
                    edge: edge.clone(),
                })
            }
        }
    }

    pub fn disconnect_edge(&mut self, ds_edge_id: EngineEdgeID) -> bool {
        if self.edge_id_to_ds_edge_id.remove(&ds_edge_id.edge_id).is_some() {
            if self.graph_helper.remove_edge(ds_edge_id.edge_id).is_ok() {
                log::trace!("Successfully disconnected edge: {:?}", ds_edge_id);
                true
            } else {
                panic!("Unexpected error while disconnecting edge in graph: {:?}", ds_edge_id);
            }
        } else {
            log::warn!(
                "Could not disconnect edge: {:?}: Edge was not found in the graph",
                ds_edge_id
            );
            false
        }
    }

    pub fn reset(&mut self, engine_timer: &mut EngineTimerWheel) {
        // Try to gracefully remove all existing plugins.
        for plugin_host in self.shared_pools.plugin_hosts.iter_mut() {
            if let Some(processor_to_drop) =
                plugin_host.schedule_remove(&self.coll_handle, engine_timer)
            {
                self.plugin_processors_to_drop.push(processor_to_drop);
            }
        }

        self.schedule_version += 1;
        self.shared_pools.shared_schedule.set_new_schedule(
            ProcessorSchedule::new_empty(
                self.max_frames as usize,
                self.shared_pools.transports.transport.clone(),
                self.plugin_processors_to_drop.drain(..).collect(),
                self.schedule_version,
            ),
            &self.coll_handle,
        );

        // TODO: Check that the process thread is still alive.
        let process_thread_is_alive = true;

        if process_thread_is_alive {
            let start_time = std::time::Instant::now();

            const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

            // Wait for all plugins to be removed.
            while !self.shared_pools.plugin_hosts.is_empty() && start_time.elapsed() < TIMEOUT {
                std::thread::sleep(std::time::Duration::from_millis(10));

                let mut _events_out: SmallVec<[OnIdleEvent; 32]> = SmallVec::new();

                let _ = self.on_idle(&mut _events_out, engine_timer);
            }

            if !self.shared_pools.plugin_hosts.is_empty() {
                log::error!("Timed out while removing all plugins");
            }
        }

        self.shared_pools.plugin_hosts.clear();
        self.shared_pools.buffers.set_num_buffers(0, 0, 0);
        self.edge_id_to_ds_edge_id.clear();

        self.graph_helper = AudioGraphHelper::new(PortType::NUM_TYPES);

        // ---  Add the graph input and graph output nodes to the graph  --------------------------

        let graph_in_node_id = self.graph_helper.add_node(0.0);
        let graph_out_node_id = self.graph_helper.add_node(0.0);

        self.graph_in_id = PluginInstanceID::_new(
            graph_in_node_id.into(),
            0,
            PluginInstanceType::GraphInput,
            Shared::clone(self.graph_in_id.rdn()),
        );
        self.graph_out_id = PluginInstanceID::_new(
            graph_out_node_id.into(),
            1,
            PluginInstanceType::GraphOutput,
            Shared::clone(self.graph_out_id.rdn()),
        );

        for i in 0..self.graph_in_num_audio_channels as u16 {
            self.graph_helper
                .add_port(graph_in_node_id, PortID(i as u32), PortType::Audio.as_type_idx(), false)
                .unwrap();
        }
        for i in 0..self.graph_out_num_audio_channels as u16 {
            self.graph_helper
                .add_port(graph_out_node_id, PortID(i as u32), PortType::Audio.as_type_idx(), true)
                .unwrap();
        }
    }

    /// Compile the audio graph into a schedule that is sent to the audio thread.
    ///
    /// If an error is returned then the graph **MUST** be restored with the previous
    /// working save state.
    pub fn compile(&mut self) -> Result<(), GraphCompilerError> {
        self.schedule_version += 1;

        match compiler::compile_graph(
            &mut self.shared_pools,
            &mut self.graph_helper,
            &self.graph_in_id,
            &self.graph_out_id,
            self.graph_in_num_audio_channels,
            self.graph_out_num_audio_channels,
            self.plugin_processors_to_drop.drain(..).collect(),
            &mut self.verifier,
            self.schedule_version,
            &self.coll_handle,
        ) {
            Ok(schedule) => {
                log::debug!("Successfully compiled new schedule:\n{:?}", &schedule);
                self.shared_pools.shared_schedule.set_new_schedule(schedule, &self.coll_handle);
                Ok(())
            }
            Err(e) => {
                // Replace the current schedule with an emtpy one now that the graph
                // is in an invalid state.
                self.shared_pools.shared_schedule.set_new_schedule(
                    ProcessorSchedule::new_empty(
                        self.max_frames as usize,
                        self.shared_pools.transports.transport.clone(),
                        self.plugin_processors_to_drop.drain(..).collect(),
                        self.schedule_version,
                    ),
                    &self.coll_handle,
                );
                Err(e)
            }
        }
    }

    pub fn collect_save_states(&mut self) -> Vec<(PluginInstanceID, PluginHostSaveState)> {
        self.shared_pools
            .plugin_hosts
            .iter_mut()
            .filter_map(|plugin_host| {
                if plugin_host.is_save_state_dirty() {
                    Some((plugin_host.id().clone(), plugin_host.collect_save_state()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn on_idle(
        &mut self,
        events_out: &mut SmallVec<[OnIdleEvent; 32]>,
        engine_timer: &mut EngineTimerWheel,
    ) -> bool {
        let mut plugins_to_remove: SmallVec<[PluginInstanceID; 4]> = SmallVec::new();
        let mut recompile_graph = false;

        for plugin_host in self.shared_pools.plugin_hosts.iter_mut() {
            let (res, modified_params, processor_to_drop) = plugin_host.on_idle(
                self.sample_rate,
                self.min_frames,
                self.max_frames,
                &self.coll_handle,
                &mut self.graph_helper,
                events_out,
                &mut self.edge_id_to_ds_edge_id,
                &self.thread_ids,
                self.schedule_version,
                engine_timer,
            );

            match res {
                OnIdleResult::Ok => {}
                OnIdleResult::PluginDeactivated => {
                    events_out.push(OnIdleEvent::PluginDeactivated {
                        plugin_id: plugin_host.id().clone(),
                        status: Ok(()),
                    });
                }
                OnIdleResult::PluginActivated(status) => {
                    recompile_graph |= status.caused_recompile;

                    events_out.push(OnIdleEvent::PluginActivated {
                        plugin_id: plugin_host.id().clone(),
                        status,
                    });
                }
                OnIdleResult::PluginReadyToRemove => {
                    plugins_to_remove.push(plugin_host.id().clone());

                    // The user is already aware of the plugin being removed since
                    // they removed it in a previous call to
                    // EngineMainThread::modify_graph()`.
                }
                OnIdleResult::PluginFailedToActivate(e) => {
                    events_out.push(OnIdleEvent::PluginDeactivated {
                        plugin_id: plugin_host.id().clone(),
                        status: Err(e),
                    });
                }
            }

            if !modified_params.is_empty() {
                events_out.push(OnIdleEvent::PluginParamsModified {
                    plugin_id: plugin_host.id().clone(),
                    modified_params: modified_params.to_owned(),
                });
            }

            if let Some(p) = processor_to_drop {
                self.plugin_processors_to_drop.push(p);
            }
        }

        for plugin in plugins_to_remove.iter() {
            self.shared_pools.plugin_hosts.remove(plugin);
        }

        recompile_graph
    }

    pub fn get_plugin_host(&self, id: &PluginInstanceID) -> Option<&PluginHostMainThread> {
        self.shared_pools.plugin_hosts.get(id)
    }

    pub fn get_plugin_host_mut(
        &mut self,
        id: &PluginInstanceID,
    ) -> Option<&mut PluginHostMainThread> {
        self.shared_pools.plugin_hosts.get_mut(id)
    }

    pub fn get_plugin_host_by_unique_id_mut(
        &mut self,
        id: u64,
    ) -> Option<&mut PluginHostMainThread> {
        self.shared_pools.plugin_hosts.get_by_unique_id_mut(id)
    }

    pub fn graph_in_id(&self) -> &PluginInstanceID {
        &self.graph_in_id
    }

    pub fn graph_out_id(&self) -> &PluginInstanceID {
        &self.graph_out_id
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EngineEdgeID {
    pub(crate) unique_id: u64,
    pub(crate) edge_id: EdgeID,
}

impl PartialEq for EngineEdgeID {
    fn eq(&self, other: &Self) -> bool {
        self.unique_id == other.unique_id
    }
}

impl Eq for EngineEdgeID {}

impl Hash for EngineEdgeID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_id.hash(state)
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: EngineEdgeID,

    pub edge_type: PortType,

    pub src_plugin_id: PluginInstanceID,
    pub dst_plugin_id: PluginInstanceID,

    pub src_port_id: u32,
    pub dst_port_id: u32,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Edge {}
