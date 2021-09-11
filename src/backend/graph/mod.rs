use atomic_refcell::AtomicRefCell;
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

use audio_graph::{NodeIdent, NodeRef};

use crate::backend::timeline::{TimelineTransport, TimelineTransportHandle};
use crate::backend::MAX_BLOCKSIZE;

pub struct GraphInterface {
    shared_graph_state: Shared<SharedCell<CompiledGraph>>,
    resource_pool: GraphResourcePool,
    graph_state: GraphState,

    sample_rate: SampleRate,
    coll_handle: Handle,

    root_node_ref: NodeRef,
}

impl GraphInterface {
    pub fn new(
        sample_rate: SampleRate,
        coll_handle: Handle,
        root_node: Box<dyn AudioGraphNode>,
    ) -> (Self, Shared<SharedCell<CompiledGraph>>, TimelineTransportHandle) {
        let collector = Collector::new();

        let (shared_graph_state, mut resource_pool, timeline_handle) =
            CompiledGraph::new(collector.handle(), sample_rate);
        let rt_shared_state = Shared::clone(&shared_graph_state);

        let mut graph_state = GraphState::new();

        let root_node_ref = graph_state.add_new_node(
            root_node.mono_audio_in_ports(),
            root_node.mono_audio_out_ports(),
            root_node.stereo_audio_in_ports(),
            root_node.stereo_audio_out_ports(),
        );

        resource_pool.add_node(root_node_ref, root_node);

        (
            Self {
                shared_graph_state,
                resource_pool,
                graph_state: GraphState::new(),
                sample_rate,
                coll_handle,
                root_node_ref,
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
            root_node_ref: self.root_node_ref,
        };

        (f)(graph_state_ref);

        self.compile_graph();
    }

    fn compile_graph(&mut self) {
        let mut tasks = Vec::<AudioGraphTask>::new();

        let master_out_buffer = if self.graph_state.node_states.is_empty() {
            // We will need at-least one stereo buffer.
            if self.resource_pool.stereo_block_buffers_f32.len() < 1 {
                self.resource_pool.add_stereo_audio_block_buffers_f32(1);
            }

            &self.resource_pool.stereo_block_buffers_f32[0]
        } else {
            let graph_schedule = self.graph_state.graph.compile(self.root_node_ref);

            for entry in graph_schedule.iter() {
                match entry.node {
                    NodeIdent::DelayComp(port_type) => {}
                    NodeIdent::User(node_ident) => {
                        let node_idx: usize = node_ident.into();

                        if let Some(Some(node)) = self.resource_pool.nodes.get(node_idx) {
                            let node = Shared::clone(node);

                            // inputs may have multiple buffers to handle.
                            for (port, buffers) in &entry.inputs {
                                for b in buffers {
                                    // ...
                                }
                            }
                            // outputs have exactly one buffer they write to
                            for (port, buf) in &entry.outputs {
                                // ...
                            }
                        } else {
                            log::error!(
                                "Compiler error occured! Node with index {} does not exist.",
                                node_idx
                            );
                        }
                    }
                }
            }

            todo!()
        };

        let new_schedule = Schedule::new(tasks, self.sample_rate, Shared::clone(master_out_buffer));

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
    root_node_ref: NodeRef,
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

    pub fn root_node_ref(&self) -> NodeRef {
        self.root_node_ref
    }

    /// Get information about the number of ports in a node.
    pub fn get_node_info(&self, node_ref: NodeRef) -> Result<&NodeState, audio_graph::Error> {
        self.graph.get_node_state(node_ref)
    }

    /// Replace a node while attempting to keep previous connections.
    pub fn replace_node(
        &mut self,
        node_ref: NodeRef,
        new_node: Box<dyn AudioGraphNode>,
    ) -> Result<(), audio_graph::Error> {
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
        if node_ref == self.root_node_ref {
            // Do not delete the root node.
            log::warn!("Caller tried to remove the root node.");
            return Ok(());
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
        // Allocate a buffer for the master output.
        resource_pool.add_stereo_audio_block_buffers_f32(1);

        let master_out = Shared::clone(&resource_pool.stereo_block_buffers_f32[0]);

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
                            master_out,
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
