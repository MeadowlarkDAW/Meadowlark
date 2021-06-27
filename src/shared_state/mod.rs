use basedrop::{Collector, Handle, Shared, SharedCell};

pub mod node;

mod graph;
mod resource_pool;
mod schedule;

pub use graph::PortType;
pub use node::AudioGraphNode;
pub use schedule::{AudioGraphTask, ProcInfo};

use graph::GraphState;
use resource_pool::ResourcePool;
use schedule::Schedule;

pub struct SharedStateManager {
    shared_state: Shared<SharedCell<SharedState>>,
    graph_state: GraphState,

    collector: Collector,

    sample_rate: f32,
}

impl SharedStateManager {
    pub fn new(
        max_audio_frames: usize,
        sample_rate: f32,
    ) -> (Self, Shared<SharedCell<SharedState>>) {
        let collector = Collector::new();

        let shared_state = SharedState::new(&collector.handle(), max_audio_frames, sample_rate);
        let rt_shared_state = Shared::clone(&shared_state);

        (
            Self {
                shared_state,
                collector,
                graph_state: GraphState::new(),
                sample_rate,
            },
            rt_shared_state,
        )
    }

    // TODO: Some way to modify the delay compensation of nodes, which will cause the graph to recompile.

    // We are using a closure for all modifications to the graph instead of using individual methods to act on
    // the graph. This is so the graph only gets compiled once after the user is done, instead of being recompiled
    // after every method.
    pub fn modify_graph<F: FnOnce(SharedGraphState<'_>, &'_ Handle)>(&mut self, f: F) {
        let mut new_resource_pool = ResourcePool::clone(&self.shared_state.get().resource_pool);

        let shared_graph_state = SharedGraphState {
            resource_pool: &mut new_resource_pool,
            graph_state: &mut self.graph_state,
            coll_handle: &self.collector.handle(),
        };

        (f)(shared_graph_state, &self.collector.handle());

        self.compile_graph(new_resource_pool);
    }

    fn compile_graph(&mut self, mut new_resource_pool: ResourcePool) {
        let mut tasks = Vec::<AudioGraphTask>::new();

        // Manually setting up the task for now. Later we will use the actual `audio_graph` crate
        // to compile the graph schedule for us.

        // We will need at-least two output buffers for the master out.
        if new_resource_pool.audio_buffers.len() < 2 {
            new_resource_pool.add_new_audio_buffers(2, &self.collector.handle());
        }

        let out_buffer_left = &new_resource_pool.audio_buffers[0];
        let out_buffer_right = &new_resource_pool.audio_buffers[0];

        // First up in the graph is the sine wave generator.
        tasks.push(AudioGraphTask::Node {
            node: Shared::clone(
                &new_resource_pool.nodes[self
                    .graph_state
                    .get_node_state(&String::from("sine_gen"))
                    .unwrap()
                    .node_pool_index],
            ),
            audio_through_buffers: vec![],
            extra_audio_in_buffers: vec![],
            extra_audio_out_buffers: vec![
                Shared::clone(out_buffer_left),
                Shared::clone(out_buffer_right),
            ],
        });

        // Next up is the gain node.
        tasks.push(AudioGraphTask::Node {
            node: Shared::clone(
                &new_resource_pool.nodes[self
                    .graph_state
                    .get_node_state(&String::from("gain"))
                    .unwrap()
                    .node_pool_index],
            ),
            audio_through_buffers: vec![
                Shared::clone(out_buffer_left),
                Shared::clone(out_buffer_right),
            ],
            extra_audio_in_buffers: vec![],
            extra_audio_out_buffers: vec![],
        });

        // Next up is the monitor node.
        tasks.push(AudioGraphTask::Node {
            node: Shared::clone(
                &new_resource_pool.nodes[self
                    .graph_state
                    .get_node_state(&String::from("monitor"))
                    .unwrap()
                    .node_pool_index],
            ),
            audio_through_buffers: vec![],
            extra_audio_in_buffers: vec![
                Shared::clone(out_buffer_left),
                Shared::clone(out_buffer_right),
            ],
            extra_audio_out_buffers: vec![],
        });

        // We are already using the master output buffers here, so we don't need to anything else.

        let new_schedule = Shared::new(
            &self.collector.handle(),
            Schedule::new(
                tasks,
                self.sample_rate,
                Shared::clone(out_buffer_left),
                Shared::clone(out_buffer_right),
            ),
        );

        let new_shared_state = Shared::new(
            &self.collector.handle(),
            SharedState {
                resource_pool: Shared::new(&self.collector.handle(), new_resource_pool),
                schedule: new_schedule,
            },
        );

        // This new state will be available to the rt thread at the top of the next process loop.
        self.shared_state.set(new_shared_state);
    }

    /// Call periodically to collect garbage in the rt thread.
    pub fn collect(&mut self) {
        self.collector.collect();
    }

    pub fn coll_handle(&self) -> Handle {
        self.collector.handle()
    }
}

pub struct SharedGraphState<'a> {
    resource_pool: &'a mut ResourcePool,
    graph_state: &'a mut GraphState,
    coll_handle: &'a Handle,
}

impl<'a> SharedGraphState<'a> {
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
        if let Err(_) = self.graph_state.add_new_node(node_id.clone(), &node) {
            return Err(node);
        };

        self.resource_pool
            .add_node(Shared::new(self.coll_handle, node));

        Ok(())
    }

    // TODO: Return custom error type.
    /// Remove a node from the graph.
    ///
    /// This will automatically remove all connections to this node as well.
    pub(super) fn remove_node(&mut self, node_id: &String) -> Result<(), ()> {
        if let Ok(index) = self.graph_state.remove_node(node_id) {
            // This shouldn't panic because the `graph_state` found a valid index.
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
        self.graph_state.add_port_connection(
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
        self.graph_state.remove_port_connection(
            port_type,
            source_node_id,
            source_node_port_id,
            dest_node_id,
            dest_node_port_id,
        )
    }
}

pub struct SharedState {
    pub resource_pool: Shared<ResourcePool>,
    pub schedule: Shared<Schedule>,
}

impl SharedState {
    fn new(
        coll_handle: &Handle,
        max_audio_frames: usize,
        sample_rate: f32,
    ) -> Shared<SharedCell<SharedState>> {
        let mut resource_pool = ResourcePool::new(max_audio_frames);
        // Allocate two buffers as the master output.
        resource_pool.add_new_audio_buffers(2, coll_handle);
        let master_out_left = Shared::clone(&resource_pool.audio_buffers[0]);
        let master_out_right = Shared::clone(&resource_pool.audio_buffers[1]);

        Shared::new(
            coll_handle,
            SharedCell::new(Shared::new(
                coll_handle,
                SharedState {
                    resource_pool: Shared::new(coll_handle, resource_pool),
                    schedule: Shared::new(
                        coll_handle,
                        Schedule::new(vec![], sample_rate, master_out_left, master_out_right),
                    ),
                },
            )),
        )
    }

    /// Where the magic happens! Only to be used by the rt thread.
    pub fn process(&mut self, frames: usize) {
        // Should not panic because the non-rt thread always clones its shared state
        // before modifying it.
        Shared::get_mut(&mut self.resource_pool)
            .unwrap()
            .clear_all_buffers(frames);
        Shared::get_mut(&mut self.schedule).unwrap().process(frames);
    }
}

impl Clone for SharedState {
    fn clone(&self) -> Self {
        Self {
            resource_pool: Shared::clone(&self.resource_pool),
            schedule: Shared::clone(&self.schedule),
        }
    }
}
