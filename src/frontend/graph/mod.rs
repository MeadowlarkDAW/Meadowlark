use fnv::FnvHashMap;
use std::any::Any;

mod node;
pub use node::{AudioGraphNode, AudioGraphNodeState, NodeConnection, NodeMessage};

pub struct AudioGraphState {
    node_states: Vec<Box<dyn Any>>,
    node_map: FnvHashMap<String, usize>,
    node_connections: Vec<NodeConnection>,

    msg_tx: llq::Producer<AudioGraphMsg>,
}

impl AudioGraphState {
    pub fn new() -> (Self, AudioGraph) {
        let (msg_tx, msg_rx) = llq::Queue::<AudioGraphMsg>::new().split();

        (
            Self {
                node_states: Vec::new(),         // TODO: Reserve capacity?
                node_map: FnvHashMap::default(), // TODO: Reserve capacity?
                node_connections: Vec::new(),    // TODO: Reserve capacity?

                msg_tx,
            },
            AudioGraph {},
        )
    }

    /// Add a new node to the audio graph. This will cause the audio graph to recompile.
    ///
    /// Returns an error if the `id` is already being used for an existing node in the graph, or
    /// if is trying to connect to a node/port that doesn't exist.
    ///
    /// TODO: Detect cycles.
    pub fn add_node<T: Any + AudioGraphNodeState>(
        &mut self,
        id: &str,
        state: Box<T>,
        add_connections: Vec<NodeConnection>,
    ) -> Result<(), ()> {
        if self.node_map.contains_key(id) {
            return Err(()); // id is already being used
        }

        self.node_states.push(state);
        self.node_map
            .insert(String::from(id), self.node_states.len() - 1);

        self.compile_graph();

        Ok(())
    }

    fn add_connections(&mut self, connections: Vec<NodeConnection>) -> Result<(), ()> {}

    /// Add multiple new nodes to the audio graph. This is more efficient that calling `add_node()`
    /// individually as it will cause the audio graph to only recomplile once.
    ///
    /// Returns an error if any of the id's are already being used for an existing node in the graph.
    pub fn add_multiple_nodes<T: Any + AudioGraphNodeState>(
        &mut self,
        nodes: Vec<(String, Box<T>)>,
    ) -> Result<(), ()> {
        // Check if any of the ids are already being used.
        for (id, _) in nodes.iter() {
            if self.node_map.contains_key(id) {
                return Err(());
            }
        }

        for (id, state) in nodes.drain(..) {
            self.node_states.push(state);
            self.node_map.insert(id, self.node_states.len() - 1);
        }

        self.compile_graph();

        Ok(())
    }

    /// Removes a node from the graph. This will cause the audio graph to recompile.
    pub fn remove_node(&mut self, id: &str) -> Result<(), ()> {}

    /// Get the state of the node with the given ID.
    ///
    /// This will return an error if the node with the `id` does not exist in the audio graph, or if
    /// the node is not the same type as the given type in `T`.
    pub fn get_node<T: Any + AudioGraphNodeState>(&self, id: &str) -> Option<&T> {
        if let Some(i) = self.node_map.get(id) {
            if let Some(n) = self.node_states[*i].downcast_ref::<T>() {
                return Some(n);
            }
        }
        None
    }

    /// Modify the node with the given ID.
    ///
    /// If any of the node's parameters were changed, then a message will be immediately sent to the
    /// rt thread alerting it of the change.
    ///
    /// If any of the node's connections were changed, then the audio graph will be recompiled.
    ///
    /// This will return an error if the node with the `id` does not exist in the audio graph, or if
    /// the node is not the same type as the given type in `T`.
    pub fn modify_node<T: Any + AudioGraphNodeState, F: FnOnce(&mut T)>(
        &mut self,
        id: &str,
        f: F,
    ) -> Result<(), ()> {
        if let Some(node_index) = self.node_map.get(id) {
            if let Some(node_state) = self.node_states[*node_index].downcast_mut::<T>() {
                (f)(node_state);

                // Check if the user modified any paramaters of the node.
                while let Some(msg) = node_state.pop_message() {
                    // TODO: Optimize by using a pool of recycled llq::Nodes?
                    self.msg_tx
                        .push(llq::Node::new(AudioGraphMsg::Node((*node_index, msg))));
                }

                return Ok(());
            }
        }
        Err(())
    }

    fn compile_graph(&mut self) {}
}

pub struct AudioGraph {}

impl AudioGraph {
    pub fn sync(&mut self) {}
}

pub enum AudioGraphMsg {
    Node((usize, NodeMessage)),
    Graph,
}
