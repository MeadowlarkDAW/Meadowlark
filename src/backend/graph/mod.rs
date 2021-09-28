use atomic_refcell::AtomicRefCell;
use audio_graph::DelayCompInfo;
use basedrop::{Collector, Handle, Shared, SharedCell};
use rusty_daw_time::SampleRate;

pub mod node;

mod audio_buffer;
mod graph_state;
mod resource_pool;
mod schedule;
mod task;

pub use audio_buffer::{MonoBlockBuffer, StereoBlockBuffer};
pub use graph_state::{NodeState, PortType};
pub use node::AudioGraphNode;
pub use schedule::ProcInfo;
pub use task::{
    AudioGraphTask, MonoProcBuffers, MonoProcBuffersMut, ProcBuffers, StereoProcBuffers,
    StereoProcBuffersMut,
};

use graph_state::GraphState;
use resource_pool::GraphResourcePool;
use schedule::Schedule;

use audio_graph::NodeRef;

use crate::backend::generic_nodes::gain::{GainNodeHandle, StereoGainNode};
use crate::backend::generic_nodes::sample_delay::{MonoSampleDelayNode, StereoSampleDelayNode};
use crate::backend::generic_nodes::sum::{MonoSumNode, StereoSumNode};
use crate::backend::graph::graph_state::PortIdent;
use crate::backend::graph::resource_pool::{DelayCompNodeKey, SumNodeKey};
use crate::backend::timeline::{TimelineTransport, TimelineTransportHandle};
use crate::backend::MAX_BLOCKSIZE;

pub struct GraphInterface {
    shared_graph_state: Shared<SharedCell<CompiledGraph>>,
    resource_pool: GraphResourcePool,
    graph_state: GraphState,

    sample_rate: SampleRate,
    coll_handle: Handle,

    root_node_ref: NodeRef,
    root_node_handle: GainNodeHandle,
}

impl GraphInterface {
    pub fn new(
        sample_rate: SampleRate,
        coll_handle: Handle,
    ) -> (Self, Shared<SharedCell<CompiledGraph>>, TimelineTransportHandle) {
        let collector = Collector::new();

        let (shared_graph_state, mut resource_pool, timeline_handle) =
            CompiledGraph::new(collector.handle(), sample_rate);
        let rt_shared_state = Shared::clone(&shared_graph_state);

        let mut graph_state = GraphState::new();

        let (root_node, root_node_handle) = StereoGainNode::new(0.0, -90.0, 12.0, sample_rate);
        let root_node_ref = graph_state.add_new_node(
            root_node.mono_audio_in_ports(),
            root_node.mono_audio_out_ports(),
            root_node.stereo_audio_in_ports(),
            root_node.stereo_audio_out_ports(),
        );

        resource_pool.add_node(root_node_ref, Box::new(root_node));

        (
            Self {
                shared_graph_state,
                resource_pool,
                graph_state,
                sample_rate,
                coll_handle,
                root_node_ref,
                root_node_handle,
            },
            rt_shared_state,
            timeline_handle,
        )
    }

    // TODO: Some way to modify the delay compensation of nodes, which will cause the graph to recompile.

    // We are using a closure for all modifications to the graph instead of using individual methods to act on
    // the graph. This is so the graph only gets compiled once after the user is done, instead of being recompiled
    // after every method.
    pub fn modify_graph<F: FnOnce(GraphStateRef<'_>)>(&mut self, f: F) {
        let graph_state_ref = GraphStateRef {
            resource_pool: &mut self.resource_pool,
            graph: &mut self.graph_state,
            root_node: self.root_node_ref,
        };

        (f)(graph_state_ref);

        self.compile_graph();
    }

    // This is one hefty boi of a function
    fn compile_graph(&mut self) {
        let mut tasks = Vec::<AudioGraphTask>::new();
        let mut master_out_buffer = None;

        // Flag all delay comp and sum nodes as unused so we can detect which ones should be
        // removed later.
        self.resource_pool.flag_unused();

        // Used to detect if there are more buffers allocated than needed.
        let mut max_mono_block_buffer_id = 0;
        let mut max_stereo_block_buffer_id = 0;
        let mut max_temp_mono_block_buffer_id = 0;
        let mut max_temp_stereo_block_buffer_id = 0;

        // TODO: We will need to ensure that none of these buffers overlap when we start using
        // a multi-threaded schedule.
        let mut next_temp_mono_block_buffer = 0;
        let mut next_temp_stereo_block_buffer = 0;

        let graph_schedule = self.graph_state.graph.compile();

        // Insert a mono delay comp node into the schedule. This returns the ID of the temp buffer used.
        let mut insert_mono_delay_comp_node = |delay_comp_info: &DelayCompInfo<
            NodeRef,
            PortIdent,
        >,
                                               node_id: NodeRef,
                                               port_id: PortIdent,
                                               buffer_id: usize|
         -> usize {
            let delayed_buffer =
                self.resource_pool.get_temp_mono_audio_block_buffer(next_temp_mono_block_buffer);
            next_temp_mono_block_buffer += 1;

            let src_node_id: usize = delay_comp_info.source_node.into();
            let dst_node_id: usize = node_id.into();
            let key = DelayCompNodeKey {
                src_node_id: src_node_id as u32,
                src_node_port: delay_comp_info.source_port,
                dst_node_id: dst_node_id as u32,
                dst_node_port: port_id,
            };

            let new_delay = delay_comp_info.delay as u32;

            let delay_node = if let Some(old_delay_node) =
                self.resource_pool.delay_comp_nodes.get_mut(&key)
            {
                // Mark that this node is still being used.
                old_delay_node.2 = true;

                if old_delay_node.1 == new_delay {
                    // Delay has not changed, just return the existing node.
                    Shared::clone(&old_delay_node.0)
                } else {
                    // Delay has changed, replace the node.
                    let new_delay_node: Box<dyn AudioGraphNode> =
                        Box::new(MonoSampleDelayNode::new(new_delay));
                    let new_node =
                        Shared::new(&self.coll_handle, AtomicRefCell::new(new_delay_node));

                    old_delay_node.0 = Shared::clone(&new_node);
                    old_delay_node.1 = new_delay;

                    new_node
                }
            } else {
                let new_delay_node: Box<dyn AudioGraphNode> =
                    Box::new(MonoSampleDelayNode::new(new_delay));
                let new_node = Shared::new(&self.coll_handle, AtomicRefCell::new(new_delay_node));

                let _ = self
                    .resource_pool
                    .delay_comp_nodes
                    .insert(key, (Shared::clone(&new_node), new_delay, true));

                new_node
            };

            tasks.push(AudioGraphTask {
                node: delay_node,
                proc_buffers: ProcBuffers {
                    mono_audio_in: MonoProcBuffers::new(vec![(
                        self.resource_pool.get_mono_audio_block_buffer(buffer_id),
                        0,
                    )]),
                    mono_audio_out: MonoProcBuffersMut::new(vec![(delayed_buffer, 0)]),
                    stereo_audio_in: StereoProcBuffers::new(vec![]),
                    stereo_audio_out: StereoProcBuffersMut::new(vec![]),
                },
            });

            next_temp_mono_block_buffer - 1
        };

        // Insert a stereo delay comp node into the schedule. This returns the ID of the temp buffer used.
        let mut insert_stereo_delay_comp_node =
            |delay_comp_info: &DelayCompInfo<NodeRef, PortIdent>,
             node_id: NodeRef,
             port_id: PortIdent,
             buffer_id: usize|
             -> usize {
                let delayed_buffer = self
                    .resource_pool
                    .get_temp_stereo_audio_block_buffer(next_temp_stereo_block_buffer);
                next_temp_stereo_block_buffer += 1;

                let src_node_id: usize = delay_comp_info.source_node.into();
                let dst_node_id: usize = node_id.into();
                let key = DelayCompNodeKey {
                    src_node_id: src_node_id as u32,
                    src_node_port: delay_comp_info.source_port,
                    dst_node_id: dst_node_id as u32,
                    dst_node_port: port_id,
                };

                let new_delay = delay_comp_info.delay as u32;

                let delay_node = if let Some(old_delay_node) =
                    self.resource_pool.delay_comp_nodes.get_mut(&key)
                {
                    // Mark that this node is still being used.
                    old_delay_node.2 = true;

                    if old_delay_node.1 == new_delay {
                        // Delay has not changed, just return the existing node.
                        Shared::clone(&old_delay_node.0)
                    } else {
                        // Delay has changed, replace the node.
                        let new_delay_node: Box<dyn AudioGraphNode> =
                            Box::new(StereoSampleDelayNode::new(new_delay));
                        let new_node =
                            Shared::new(&self.coll_handle, AtomicRefCell::new(new_delay_node));

                        old_delay_node.0 = Shared::clone(&new_node);
                        old_delay_node.1 = new_delay;

                        new_node
                    }
                } else {
                    let new_delay_node: Box<dyn AudioGraphNode> =
                        Box::new(StereoSampleDelayNode::new(new_delay));
                    let new_node =
                        Shared::new(&self.coll_handle, AtomicRefCell::new(new_delay_node));

                    let _ = self
                        .resource_pool
                        .delay_comp_nodes
                        .insert(key, (Shared::clone(&new_node), new_delay, true));

                    new_node
                };

                tasks.push(AudioGraphTask {
                    node: delay_node,
                    proc_buffers: ProcBuffers {
                        mono_audio_in: MonoProcBuffers::new(vec![]),
                        mono_audio_out: MonoProcBuffersMut::new(vec![]),
                        stereo_audio_in: StereoProcBuffers::new(vec![(
                            self.resource_pool.get_stereo_audio_block_buffer(buffer_id),
                            0,
                        )]),
                        stereo_audio_out: StereoProcBuffersMut::new(vec![(delayed_buffer, 0)]),
                    },
                });

                next_temp_stereo_block_buffer - 1
            };

        for entry in graph_schedule.iter() {
            let mut mono_audio_in: Vec<(Shared<AtomicRefCell<MonoBlockBuffer<f32>>>, usize)> =
                Vec::new();
            let mut mono_audio_out: Vec<(Shared<AtomicRefCell<MonoBlockBuffer<f32>>>, usize)> =
                Vec::new();
            let mut stereo_audio_in: Vec<(Shared<AtomicRefCell<StereoBlockBuffer<f32>>>, usize)> =
                Vec::new();
            let mut stereo_audio_out: Vec<(Shared<AtomicRefCell<StereoBlockBuffer<f32>>>, usize)> =
                Vec::new();

            // TODO: We will need to ensure that none of these buffers overlap when we start using
            // a multi-threaded schedule.
            next_temp_mono_block_buffer = 0;
            next_temp_stereo_block_buffer = 0;

            for (port_ident, buffers) in entry.inputs.iter() {
                if buffers.len() == 1 {
                    // Summing is not needed

                    let (buf, delay_comp) = buffers[0];

                    let buffer_id = buf.buffer_id;
                    match port_ident.port_type {
                        PortType::MonoAudio => {
                            if buffer_id > max_mono_block_buffer_id {
                                max_mono_block_buffer_id = buffer_id;
                            }

                            let buffer = if let Some(delay_comp_info) = &delay_comp {
                                // Delay compensation needed
                                let temp_buffer_id = insert_mono_delay_comp_node(
                                    delay_comp_info,
                                    entry.node,
                                    *port_ident,
                                    buffer_id,
                                );

                                self.resource_pool.get_temp_mono_audio_block_buffer(temp_buffer_id)
                            } else {
                                // No delay compensation needed
                                self.resource_pool.get_mono_audio_block_buffer(buffer_id)
                            };

                            mono_audio_in.push((buffer, usize::from(port_ident.index)));
                        }
                        PortType::StereoAudio => {
                            if buffer_id > max_stereo_block_buffer_id {
                                max_stereo_block_buffer_id = buffer_id;
                            }

                            let buffer = if let Some(delay_comp_info) = &delay_comp {
                                // Delay compensation needed
                                let temp_buffer_id = insert_stereo_delay_comp_node(
                                    delay_comp_info,
                                    entry.node,
                                    *port_ident,
                                    buffer_id,
                                );

                                self.resource_pool
                                    .get_temp_stereo_audio_block_buffer(temp_buffer_id)
                            } else {
                                // No delay compensation needed
                                self.resource_pool.get_stereo_audio_block_buffer(buffer_id)
                            };

                            stereo_audio_in.push((buffer, usize::from(port_ident.index)));
                        }
                    }
                } else {
                    let node_id: usize = entry.node.into();
                    let num_inputs = buffers.len() as u32;
                    match port_ident.port_type {
                        PortType::MonoAudio => {
                            let mut sum_mono_audio_in: Vec<(
                                Shared<AtomicRefCell<MonoBlockBuffer<f32>>>,
                                usize,
                            )> = Vec::with_capacity(buffers.len());

                            for (buf, delay_comp) in buffers.iter() {
                                let buffer_id = buf.buffer_id;
                                if buffer_id > max_mono_block_buffer_id {
                                    max_mono_block_buffer_id = buffer_id;
                                }

                                let buffer = if let Some(delay_comp_info) = &delay_comp {
                                    // Delay compensation needed
                                    let temp_buffer_id = insert_mono_delay_comp_node(
                                        delay_comp_info,
                                        entry.node,
                                        *port_ident,
                                        buffer_id,
                                    );

                                    self.resource_pool
                                        .get_temp_mono_audio_block_buffer(temp_buffer_id)
                                } else {
                                    // No delay compensation needed
                                    self.resource_pool.get_mono_audio_block_buffer(buffer_id)
                                };

                                sum_mono_audio_in.push((buffer, usize::from(port_ident.index)));
                            }

                            let temp_buffer = self
                                .resource_pool
                                .get_temp_mono_audio_block_buffer(next_temp_mono_block_buffer);
                            next_temp_mono_block_buffer += 1;

                            let key = SumNodeKey { node_id: node_id as u32, port: *port_ident };

                            let sum_node = if let Some(old_sum_node) =
                                self.resource_pool.sum_nodes.get_mut(&key)
                            {
                                // Mark that this node is still being used.
                                old_sum_node.2 = true;

                                if old_sum_node.1 == num_inputs {
                                    // Number of inputs has not changed, just return the existing node.
                                    Shared::clone(&old_sum_node.0)
                                } else {
                                    // Number of inputs has changed, replace the node.
                                    let new_sum_node: Box<dyn AudioGraphNode> =
                                        Box::new(MonoSumNode::new(num_inputs));
                                    let new_node = Shared::new(
                                        &self.coll_handle,
                                        AtomicRefCell::new(new_sum_node),
                                    );

                                    old_sum_node.0 = Shared::clone(&new_node);
                                    old_sum_node.1 = num_inputs;

                                    new_node
                                }
                            } else {
                                let new_sum_node: Box<dyn AudioGraphNode> =
                                    Box::new(MonoSumNode::new(num_inputs));
                                let new_node = Shared::new(
                                    &self.coll_handle,
                                    AtomicRefCell::new(new_sum_node),
                                );

                                let _ = self
                                    .resource_pool
                                    .sum_nodes
                                    .insert(key, (Shared::clone(&new_node), num_inputs, true));

                                new_node
                            };

                            tasks.push(AudioGraphTask {
                                node: sum_node,
                                proc_buffers: ProcBuffers {
                                    mono_audio_in: MonoProcBuffers::new(sum_mono_audio_in),
                                    mono_audio_out: MonoProcBuffersMut::new(vec![(
                                        Shared::clone(&temp_buffer),
                                        0,
                                    )]),
                                    stereo_audio_in: StereoProcBuffers::new(vec![]),
                                    stereo_audio_out: StereoProcBuffersMut::new(vec![]),
                                },
                            });

                            mono_audio_in.push((temp_buffer, usize::from(port_ident.index)));
                        }
                        PortType::StereoAudio => {
                            let mut sum_stereo_audio_in: Vec<(
                                Shared<AtomicRefCell<StereoBlockBuffer<f32>>>,
                                usize,
                            )> = Vec::with_capacity(buffers.len());

                            for (buf, delay_comp) in buffers.iter() {
                                let buffer_id = buf.buffer_id;
                                if buffer_id > max_stereo_block_buffer_id {
                                    max_stereo_block_buffer_id = buffer_id;
                                }

                                let buffer = if let Some(delay_comp_info) = &delay_comp {
                                    // Delay compensation needed
                                    let temp_buffer_id = insert_stereo_delay_comp_node(
                                        delay_comp_info,
                                        entry.node,
                                        *port_ident,
                                        buffer_id,
                                    );

                                    self.resource_pool
                                        .get_temp_stereo_audio_block_buffer(temp_buffer_id)
                                } else {
                                    // No delay compensation needed
                                    self.resource_pool.get_stereo_audio_block_buffer(buffer_id)
                                };

                                sum_stereo_audio_in.push((buffer, usize::from(port_ident.index)));
                            }

                            let temp_buffer = self
                                .resource_pool
                                .get_temp_stereo_audio_block_buffer(next_temp_stereo_block_buffer);
                            next_temp_stereo_block_buffer += 1;

                            let key = SumNodeKey { node_id: node_id as u32, port: *port_ident };

                            let sum_node = if let Some(old_sum_node) =
                                self.resource_pool.sum_nodes.get_mut(&key)
                            {
                                // Mark that this node is still being used.
                                old_sum_node.2 = true;

                                if old_sum_node.1 == num_inputs {
                                    // Number of inputs has not changed, just return the existing node.
                                    Shared::clone(&old_sum_node.0)
                                } else {
                                    // Number of inputs has changed, replace the node.
                                    let new_sum_node: Box<dyn AudioGraphNode> =
                                        Box::new(StereoSumNode::new(num_inputs));
                                    let new_node = Shared::new(
                                        &self.coll_handle,
                                        AtomicRefCell::new(new_sum_node),
                                    );

                                    old_sum_node.0 = Shared::clone(&new_node);
                                    old_sum_node.1 = num_inputs;

                                    new_node
                                }
                            } else {
                                let new_sum_node: Box<dyn AudioGraphNode> =
                                    Box::new(StereoSumNode::new(num_inputs));
                                let new_node = Shared::new(
                                    &self.coll_handle,
                                    AtomicRefCell::new(new_sum_node),
                                );

                                let _ = self
                                    .resource_pool
                                    .sum_nodes
                                    .insert(key, (Shared::clone(&new_node), num_inputs, true));

                                new_node
                            };

                            tasks.push(AudioGraphTask {
                                node: sum_node,
                                proc_buffers: ProcBuffers {
                                    mono_audio_in: MonoProcBuffers::new(vec![]),
                                    mono_audio_out: MonoProcBuffersMut::new(vec![]),
                                    stereo_audio_in: StereoProcBuffers::new(sum_stereo_audio_in),
                                    stereo_audio_out: StereoProcBuffersMut::new(vec![(
                                        Shared::clone(&temp_buffer),
                                        0,
                                    )]),
                                },
                            });

                            stereo_audio_in.push((temp_buffer, usize::from(port_ident.index)));
                        }
                    }
                }
            }

            if next_temp_mono_block_buffer != 0 {
                if next_temp_mono_block_buffer - 1 > max_temp_mono_block_buffer_id {
                    max_temp_mono_block_buffer_id = next_temp_mono_block_buffer - 1;
                }
            }
            if next_temp_stereo_block_buffer != 0 {
                if next_temp_stereo_block_buffer - 1 > max_temp_stereo_block_buffer_id {
                    max_temp_stereo_block_buffer_id = next_temp_stereo_block_buffer - 1;
                }
            }

            let node_id: usize = entry.node.into();
            let mut found = false;
            if let Some(node) = self.resource_pool.nodes.get(node_id) {
                if let Some(node) = node {
                    found = true;

                    if entry.node == self.root_node_ref {
                        if let Some(buffer) = stereo_audio_out.get(0) {
                            master_out_buffer = Some(Shared::clone(&buffer.0));
                        }
                    }

                    tasks.push(AudioGraphTask {
                        node: Shared::clone(node),
                        proc_buffers: ProcBuffers {
                            mono_audio_in: MonoProcBuffers::new(mono_audio_in),
                            mono_audio_out: MonoProcBuffersMut::new(mono_audio_out),
                            stereo_audio_in: StereoProcBuffers::new(stereo_audio_in),
                            stereo_audio_out: StereoProcBuffersMut::new(stereo_audio_out),
                        },
                    });
                }
            }

            if !found {
                log::error!("Schedule error: Node with ID {} does not exist", node_id);
                debug_assert!(false, "Schedule error: Node with ID {} does not exist", node_id);
            }
        }

        let master_out_buffer = if let Some(buffer) = master_out_buffer.take() {
            buffer
        } else {
            log::error!("No master output buffer exists. This will only output silence.");
            debug_assert!(false, "No master output buffer exists. This will only output silence.");

            max_stereo_block_buffer_id += 1;
            self.resource_pool.get_temp_stereo_audio_block_buffer(max_stereo_block_buffer_id)
        };

        // Remove buffers that are no longer needed
        if self.resource_pool.mono_block_buffers.len() > max_mono_block_buffer_id {
            self.resource_pool.remove_mono_block_buffers(
                self.resource_pool.mono_block_buffers.len() - (max_mono_block_buffer_id + 1),
            );
        }
        if self.resource_pool.stereo_block_buffers.len() > max_stereo_block_buffer_id {
            self.resource_pool.remove_stereo_block_buffers(
                self.resource_pool.stereo_block_buffers.len() - (max_stereo_block_buffer_id + 1),
            );
        }
        if self.resource_pool.temp_mono_block_buffers.len() > max_temp_mono_block_buffer_id {
            self.resource_pool.remove_temp_mono_block_buffers(
                self.resource_pool.temp_mono_block_buffers.len()
                    - (max_temp_mono_block_buffer_id + 1),
            );
        }
        if self.resource_pool.temp_stereo_block_buffers.len() > max_temp_stereo_block_buffer_id {
            self.resource_pool.remove_temp_stereo_block_buffers(
                self.resource_pool.temp_stereo_block_buffers.len()
                    - (max_temp_stereo_block_buffer_id + 1),
            );
        }

        // Remove delay comp and sum nodes that are no longer needed
        self.resource_pool.drop_unused();

        // Create the new schedule and replace the old one

        let new_schedule = Schedule::new(tasks, self.sample_rate, master_out_buffer);

        let new_shared_state = Shared::new(
            &self.coll_handle,
            CompiledGraph {
                resource_pool: AtomicRefCell::new(GraphResourcePool::clone(&self.resource_pool)),
                schedule: AtomicRefCell::new(new_schedule),
                timeline_transport: Shared::clone(
                    &self.shared_graph_state.get().timeline_transport,
                ),
            },
        );

        // This new state will be available to the rt thread at the top of the next process loop.
        self.shared_graph_state.set(new_shared_state);
    }
}

pub struct GraphStateRef<'a> {
    resource_pool: &'a mut GraphResourcePool,
    graph: &'a mut GraphState,
    root_node: NodeRef,
}

impl<'a> GraphStateRef<'a> {
    pub fn add_new_node(&mut self, node: Box<dyn AudioGraphNode>) -> NodeRef {
        let node_ref = self.graph.add_new_node(
            node.mono_audio_in_ports(),
            node.mono_audio_out_ports(),
            node.stereo_audio_in_ports(),
            node.stereo_audio_out_ports(),
        );

        self.resource_pool.add_node(node_ref, node);

        node_ref
    }

    /// Get information about the number of ports in a node.
    pub fn get_node_info(&self, node_ref: NodeRef) -> Result<&NodeState, audio_graph::Error> {
        self.graph.get_node_state(node_ref)
    }

    /// Get the root node.
    pub fn root_node(&self) -> NodeRef {
        self.root_node
    }

    /// Replace a node while attempting to keep previous connections.
    pub fn replace_node(
        &mut self,
        node_ref: NodeRef,
        new_node: Box<dyn AudioGraphNode>,
    ) -> Result<(), audio_graph::Error> {
        // Don't allow replacing the root now.
        if node_ref == self.root_node {
            return Err(audio_graph::Error::NodeDoesNotExist);
        }

        self.graph.set_num_ports(
            node_ref,
            new_node.mono_audio_in_ports(),
            new_node.mono_audio_out_ports(),
            new_node.stereo_audio_in_ports(),
            new_node.stereo_audio_out_ports(),
        )?;

        self.resource_pool.remove_node(node_ref);
        self.resource_pool.add_node(node_ref, new_node);

        Ok(())
    }

    /// Remove a node from the graph.
    ///
    /// This will automatically remove all connections to this node as well.
    ///
    /// Please note that if this call was successful, then the given `node_ref` is now
    /// invalid and must be discarded.
    pub fn remove_node(&mut self, node_ref: NodeRef) -> Result<(), audio_graph::Error> {
        // Don't allow removing the root now.
        if node_ref == self.root_node {
            return Err(audio_graph::Error::NodeDoesNotExist);
        }

        self.graph.remove_node(node_ref)?;
        self.resource_pool.remove_node(node_ref);

        Ok(())
    }

    /// Add a connection between nodes.
    pub fn connect_ports(
        &mut self,
        port_type: PortType,
        source_node_ref: NodeRef,
        source_node_port_index: usize,
        dest_node_ref: NodeRef,
        dest_node_port_index: usize,
    ) -> Result<(), audio_graph::Error> {
        self.graph.connect_ports(
            port_type,
            source_node_ref,
            source_node_port_index,
            dest_node_ref,
            dest_node_port_index,
        )
    }

    /// Remove a connection between nodes.
    pub fn disconnect_ports(
        &mut self,
        port_type: PortType,
        source_node_ref: NodeRef,
        source_node_port_index: usize,
        dest_node_ref: NodeRef,
        dest_node_port_index: usize,
    ) -> Result<(), audio_graph::Error> {
        self.graph.disconnect_ports(
            port_type,
            source_node_ref,
            source_node_port_index,
            dest_node_ref,
            dest_node_port_index,
        )
    }
}

pub struct CompiledGraph {
    pub resource_pool: AtomicRefCell<GraphResourcePool>,
    pub schedule: AtomicRefCell<Schedule>,
    timeline_transport: Shared<AtomicRefCell<TimelineTransport>>,
}

impl CompiledGraph {
    fn new(
        coll_handle: Handle,
        sample_rate: SampleRate,
    ) -> (Shared<SharedCell<CompiledGraph>>, GraphResourcePool, TimelineTransportHandle) {
        let mut resource_pool = GraphResourcePool::new(coll_handle.clone());

        let master_out_buffer = resource_pool.get_temp_stereo_audio_block_buffer(0);

        let (timeline, timeline_handle) = TimelineTransport::new(coll_handle.clone(), sample_rate);

        (
            Shared::new(
                &coll_handle,
                SharedCell::new(Shared::new(
                    &coll_handle,
                    CompiledGraph {
                        resource_pool: AtomicRefCell::new(GraphResourcePool::clone(&resource_pool)),
                        schedule: AtomicRefCell::new(Schedule::new(
                            vec![],
                            sample_rate,
                            master_out_buffer,
                        )),
                        timeline_transport: Shared::new(&coll_handle, AtomicRefCell::new(timeline)),
                    },
                )),
            ),
            resource_pool,
            timeline_handle,
        )
    }

    /// Where the magic happens! Only to be used by the rt thread.
    pub fn process<T: cpal::Sample>(&self, mut cpal_out: &mut [T]) {
        // Should not panic because the non-rt thread only mutates its own clone of this resource pool. It sends
        // a clone to the rt thread via a SharedCell.
        let resource_pool = &mut *AtomicRefCell::borrow_mut(&self.resource_pool);

        // Should not panic because the non-rt thread always creates a new schedule every time before sending
        // it to the rt thread via a SharedCell.
        let schedule = &mut *AtomicRefCell::borrow_mut(&self.schedule);

        // Assume output is stereo for now.
        let mut frames_left = cpal_out.len() / 2;

        // Process in blocks.
        while frames_left > 0 {
            let frames = frames_left.min(MAX_BLOCKSIZE);

            resource_pool.clear_all_buffers(frames);

            // Update the timeline transport. This should not panic because this is the only place
            // this is ever borrowed.
            let mut timeline_transport = AtomicRefCell::borrow_mut(&self.timeline_transport);
            timeline_transport.update(frames);

            schedule.process(frames, &mut timeline_transport);

            schedule.copy_master_output_to_cpal(&mut cpal_out[0..(frames * 2)]);

            cpal_out = &mut cpal_out[(frames * 2)..];
            frames_left -= frames;
        }
    }
}
