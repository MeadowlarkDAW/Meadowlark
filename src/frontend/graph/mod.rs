use basedrop::Collector;
use fnv::FnvHashMap;
use std::any::Any;

mod node;
pub use node::{AudioGraphNode, AudioGraphNodeState, NodeMessage};

pub struct AudioGraphState {
    collector: Collector,
    node_states: Vec<Box<dyn Any>>,
    node_map: FnvHashMap<String, usize>,
    dirty_nodes: Vec<String>,
    graph_changed: bool,

    msg_tx: llq::Producer<AudioGraphMsg>,
}

impl AudioGraphState {
    pub fn new() -> (Self, AudioGraph) {
        let (msg_tx, msg_rx) = llq::Queue::<AudioGraphMsg>::new().split();

        (
            Self {
                collector: Collector::new(),
                node_states: Vec::new(),
                node_map: FnvHashMap::default(),
                dirty_nodes: Vec::new(),
                graph_changed: false,

                msg_tx,
            },
            AudioGraph {},
        )
    }

    pub fn collect(&mut self) {
        self.collector.collect();
    }

    pub fn add_node<T: Any + AudioGraphNodeState>(
        &mut self,
        id: &str,
        state: Box<T>,
    ) -> Result<(), ()> {
        if self.node_map.contains_key(id) {
            return Err(()); // id is already being used
        }

        self.node_states.push(state);
        self.node_map
            .insert(String::from(id), self.node_states.len() - 1);

        self.graph_changed = true;
        Ok(())
    }

    /// Get the state of the node with the given ID
    pub fn get_node<T: Any + AudioGraphNodeState>(&self, id: &str) -> Option<&T> {
        if let Some(i) = self.node_map.get(id) {
            if let Some(n) = self.node_states[*i].downcast_ref::<T>() {
                return Some(n);
            }
        }
        None
    }

    pub fn modify_node<T: Any + AudioGraphNodeState, F: FnOnce(&mut T)>(
        &mut self,
        id: &str,
        f: F,
    ) -> Result<(), ()> {
        if let Some(i) = self.node_map.get(id) {
            if let Some(n) = self.node_states[*i].downcast_mut::<T>() {
                (f)(n);

                // Check if the user modified the connections of the node
                if n.connections_changed() {
                    self.graph_changed = true;
                }

                // Check if the user modified any paramaters of the node
                if n.parameters_changed() {
                    self.dirty_nodes.push(String::from(id));
                }

                return Ok(());
            }
        }
        Err(())
    }

    pub fn flush(&mut self) {
        if self.graph_changed {
            // Recompile the graph

            self.graph_changed = false;
        }

        // Always send node messages after graph changes in case any graph changes cause the indexes of nodes to be
        // deleted or changed

        for node_id in self.dirty_nodes.iter() {
            if let Some(node_index) = self.node_map.get(node_id) {
                let node_state = self.node_states[*node_index]
                    .downcast_mut::<&dyn AudioGraphNodeState>()
                    .unwrap();

                while let Some(msg) = node_state.pop_message() {
                    // TODO: Optimize by using a pool of recycled llq::Nodes?
                    self.msg_tx
                        .push(llq::Node::new(AudioGraphMsg::Node((*node_index, msg))));
                }
            }
        }

        self.dirty_nodes.clear();
    }
}

pub struct AudioGraph {}

impl AudioGraph {
    pub fn sync(&mut self) {}
}

pub enum AudioGraphMsg {
    Node((usize, NodeMessage)),
    Graph,
}
