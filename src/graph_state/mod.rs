use atomic_refcell::AtomicRefCell;
use basedrop::{Collector, Handle, Shared, SharedCell};

pub mod node;

mod graph;
mod resource_pool;
mod schedule;

pub use graph::PortType;
pub use node::AudioGraphNode;
pub use resource_pool::{MonoAudioPortBuffer, StereoAudioPortBuffer};
pub use schedule::{AudioGraphTask, ProcInfo};

use graph::Graph;
use resource_pool::ResourcePool;
use schedule::Schedule;

pub struct GraphStateManager {
    shared_graph_state: Shared<SharedCell<GraphState>>,
    resource_pool_state: ResourcePool,
    graph: Graph,

    collector: Collector,

    sample_rate: f32,
}

impl GraphStateManager {
    pub fn new(
        max_audio_frames: usize,
        sample_rate: f32,
    ) -> (Self, Shared<SharedCell<GraphState>>) {
        let collector = Collector::new();

        let (shared_graph_state, resource_pool_state) =
            GraphState::new(collector.handle(), max_audio_frames, sample_rate);
        let rt_shared_state = Shared::clone(&shared_graph_state);

        (
            Self {
                shared_graph_state,
                resource_pool_state,
                collector,
                graph: Graph::new(),
                sample_rate,
            },
            rt_shared_state,
        )
    }

    // TODO: Some way to modify the delay compensation of nodes, which will cause the graph to recompile.

    // We are using a closure for all modifications to the graph instead of using individual methods to act on
    // the graph. This is so the graph only gets compiled once after the user is done, instead of being recompiled
    // after every method.
    pub fn modify_graph<F: FnOnce(GraphStateRef<'_>)>(&mut self, f: F) {
        let graph_state_ref = GraphStateRef {
            resource_pool: &mut self.resource_pool_state,
            graph: &mut self.graph,
        };

        (f)(graph_state_ref);

        self.compile_graph();
    }

    fn compile_graph(&mut self) {
        let mut tasks = Vec::<AudioGraphTask>::new();

        // Manually setting up the task for now. Later we will use the actual `audio_graph` crate
        // to compile the graph schedule for us.

        // We will need at-least two stereo buffers.
        if self.resource_pool_state.stereo_audio_buffers.len() < 2 {
            self.resource_pool_state.add_stereo_audio_port_buffers(
                2 - self.resource_pool_state.stereo_audio_buffers.len(),
            );
        }

        let buffer_1 = &self.resource_pool_state.stereo_audio_buffers[0];
        let buffer_2 = &self.resource_pool_state.stereo_audio_buffers[1];

        // First up in the graph is the sine wave generator.
        tasks.push(AudioGraphTask::Node {
            node: Shared::clone(
                &self.resource_pool_state.nodes[self
                    .graph
                    .get_node_state(&String::from("sine_gen"))
                    .unwrap()
                    .node_pool_index],
            ),
            mono_audio_in_buffers: vec![],
            mono_audio_out_buffers: vec![],
            stereo_audio_in_buffers: vec![],
            stereo_audio_out_buffers: vec![Shared::clone(buffer_1)],
        });

        // Next up is the gain node.
        tasks.push(AudioGraphTask::Node {
            node: Shared::clone(
                &self.resource_pool_state.nodes[self
                    .graph
                    .get_node_state(&String::from("gain"))
                    .unwrap()
                    .node_pool_index],
            ),
            mono_audio_in_buffers: vec![],
            mono_audio_out_buffers: vec![],
            stereo_audio_in_buffers: vec![Shared::clone(buffer_1)],
            stereo_audio_out_buffers: vec![Shared::clone(buffer_2)],
        });

        // Next up is the monitor node.
        tasks.push(AudioGraphTask::Node {
            node: Shared::clone(
                &self.resource_pool_state.nodes[self
                    .graph
                    .get_node_state(&String::from("monitor"))
                    .unwrap()
                    .node_pool_index],
            ),
            mono_audio_in_buffers: vec![],
            mono_audio_out_buffers: vec![],
            stereo_audio_in_buffers: vec![Shared::clone(buffer_2)],
            stereo_audio_out_buffers: vec![Shared::clone(buffer_1)],
        });

        // We happened to end up on buffer_1 (master_out), so we don't need to do any more copying.

        let new_schedule = Schedule::new(tasks, self.sample_rate, Shared::clone(buffer_1));

        let new_shared_state = Shared::new(
            &self.collector.handle(),
            GraphState {
                resource_pool: AtomicRefCell::new(ResourcePool::clone(&self.resource_pool_state)),
                schedule: AtomicRefCell::new(new_schedule),
            },
        );

        // This new state will be available to the rt thread at the top of the next process loop.
        self.shared_graph_state.set(new_shared_state);
    }

    /// Call periodically to collect garbage in the rt thread.
    ///
    /// TODO: Actually do this!
    pub fn collect(&mut self) {
        self.collector.collect();
    }

    pub fn coll_handle(&self) -> Handle {
        self.collector.handle()
    }
}

pub struct GraphStateRef<'a> {
    resource_pool: &'a mut ResourcePool,
    graph: &'a mut Graph,
}

impl<'a> GraphStateRef<'a> {
    // TODO: Return custom error type.
    /// Add a new node to the graph.
    ///
    /// Every node must have a unique `node_id`. This will return an error if trying
    /// to create a node with an existing ID in the graph.
    pub fn add_new_node(
        &mut self,
        node_id: &String,
        node: Box<dyn AudioGraphNode>,
    ) -> Result<(), Box<dyn AudioGraphNode>> {
        if let Err(_) = self.graph.add_new_node(node_id.clone(), &node) {
            return Err(node);
        };

        self.resource_pool.add_node(node);

        Ok(())
    }

    // TODO: Return custom error type.
    /// Remove a node from the graph.
    ///
    /// This will automatically remove all connections to this node as well.
    pub(super) fn remove_node(&mut self, node_id: &String) -> Result<(), ()> {
        if let Ok(index) = self.graph.remove_node(node_id) {
            // This shouldn't panic because the `graph` found a valid index.
            self.resource_pool.remove_node(index).unwrap();

            Ok(())
        } else {
            Err(())
        }
    }

    // TODO: Return custom error type.
    /// Add a connection between nodes.
    pub(super) fn add_port_connection(
        &mut self,
        port_type: PortType,
        source_node_id: &String,
        source_node_port_id: usize,
        dest_node_id: &String,
        dest_node_port_id: usize,
    ) -> Result<(), ()> {
        self.graph.add_port_connection(
            port_type,
            source_node_id,
            source_node_port_id,
            dest_node_id,
            dest_node_port_id,
        )
    }

    // TODO: Return custom error type.
    /// Remove a connection between nodes.
    pub(super) fn remove_port_connection(
        &mut self,
        port_type: PortType,
        source_node_id: &String,
        source_node_port_id: usize,
        dest_node_id: &String,
        dest_node_port_id: usize,
    ) -> Result<(), ()> {
        self.graph.remove_port_connection(
            port_type,
            source_node_id,
            source_node_port_id,
            dest_node_id,
            dest_node_port_id,
        )
    }
}

pub struct GraphState {
    pub resource_pool: AtomicRefCell<ResourcePool>,
    pub schedule: AtomicRefCell<Schedule>,
}

impl GraphState {
    fn new(
        coll_handle: Handle,
        max_audio_frames: usize,
        sample_rate: f32,
    ) -> (Shared<SharedCell<GraphState>>, ResourcePool) {
        let mut resource_pool = ResourcePool::new(max_audio_frames, coll_handle.clone());
        // Allocate a buffer for the master output.
        resource_pool.add_stereo_audio_port_buffers(1);

        let master_out = Shared::clone(&resource_pool.stereo_audio_buffers[0]);

        (
            Shared::new(
                &coll_handle,
                SharedCell::new(Shared::new(
                    &coll_handle,
                    GraphState {
                        resource_pool: AtomicRefCell::new(ResourcePool::clone(&resource_pool)),
                        schedule: AtomicRefCell::new(Schedule::new(
                            vec![],
                            sample_rate,
                            master_out,
                        )),
                    },
                )),
            ),
            resource_pool,
        )
    }

    /// Where the magic happens! Only to be used by the rt thread.
    pub fn process<T: cpal::Sample>(&self, frames: usize, cpal_out: &mut [T]) {
        // Should not panic because the non-rt thread only mutates its own copy of these resources. It sends
        // a copy to the rt thread via a SharedCell.
        (&mut *AtomicRefCell::borrow_mut(&self.resource_pool)).clear_and_resize_all_buffers(frames);

        // Should not panic because the non-rt thread always creates a new schedule every time before sending
        // it to the rt thread via a SharedCell.
        let schedule = &mut *AtomicRefCell::borrow_mut(&self.schedule);

        schedule.process(frames);

        schedule.copy_master_output_to_cpal(cpal_out);
    }
}
