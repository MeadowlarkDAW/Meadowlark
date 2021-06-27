use fnv::FnvHashMap;

use super::node::AudioGraphNode;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortType {
    Audio,
}

#[derive(Clone)]
pub struct NodeConnection {
    pub source_node_id: String,
    pub dest_node_id: String,
    pub source_port_id: usize,
    pub dest_port_id: usize,
    pub port_type: PortType,
}

pub struct NodeState {
    pub n_audio_in_ports: usize,
    pub n_audio_out_ports: usize,

    pub self_connected_to: Vec<NodeConnection>,
    pub connected_to_self: Vec<NodeConnection>,

    pub(super) node_pool_index: usize,
}

pub struct GraphState {
    node_map: FnvHashMap<String, NodeState>,
}

impl GraphState {
    pub(super) fn new() -> Self {
        Self {
            node_map: FnvHashMap::default(),
        }
    }

    pub fn get_node_state(&self, node_id: &String) -> Option<&NodeState> {
        self.node_map.get(node_id)
    }

    // TODO: Return custom error type.
    pub(super) fn add_new_node(
        &mut self,
        node_id: String,
        node: &Box<dyn AudioGraphNode>,
    ) -> Result<(), ()> {
        // Check that node ID does not already exist.
        if self.node_map.contains_key(&node_id) {
            return Err(());
        }

        let n_audio_in_ports = node.audio_through_ports() + node.extra_audio_in_ports();
        let n_audio_out_ports = node.audio_through_ports() + node.extra_audio_out_ports();

        self.node_map.insert(
            node_id,
            NodeState {
                node_pool_index: self.node_map.len(),

                n_audio_in_ports,
                n_audio_out_ports,

                self_connected_to: Vec::new(),
                connected_to_self: Vec::new(),
            },
        );

        Ok(())
    }

    // TODO: Return custom error type.
    pub(super) fn remove_node(&mut self, node_id: &String) -> Result<usize, ()> {
        // Get around borrow checker.
        let (self_connected_to, connected_to_self, node_pool_index) = {
            let node_state = if let Some(n) = self.node_map.get(node_id) {
                n
            } else {
                return Err(());
            };
            (
                node_state.self_connected_to.clone(),
                node_state.connected_to_self.clone(),
                node_state.node_pool_index,
            )
        };

        // Remove all existing connections to this node.
        for self_connected_to in self_connected_to.iter() {
            let n_state = self
                .node_map
                .get_mut(&self_connected_to.dest_node_id)
                .unwrap();
            n_state
                .connected_to_self
                .retain(|n| &n.source_node_id != node_id);
        }
        for connected_to_self in connected_to_self.iter() {
            let n_state = self
                .node_map
                .get_mut(&connected_to_self.source_node_id)
                .unwrap();
            n_state
                .self_connected_to
                .retain(|n| &n.dest_node_id != node_id);
        }

        // Decrement the node pool index by one on all nodes that appear after this one.
        for (_, n) in self.node_map.iter_mut() {
            if n.node_pool_index > node_pool_index {
                n.node_pool_index -= 1;
            }
        }

        // Remove this node.
        let _ = self.node_map.remove(node_id);

        Ok(node_pool_index)
    }

    // TODO: Return custom error type.
    pub(super) fn add_port_connection(
        &mut self,
        port_type: PortType,
        source_node_id: &String,
        source_node_port_id: usize,
        dest_node_id: &String,
        dest_node_port_id: usize,
    ) -> Result<(), ()> {
        // TODO: Detect cycles.

        let source_node = if let Some(n) = self.node_map.get(source_node_id) {
            n
        } else {
            return Err(());
        };
        let dest_node = if let Some(n) = self.node_map.get(dest_node_id) {
            n
        } else {
            return Err(());
        };

        // Check that both nodes are different.
        if source_node_id == dest_node_id {
            return Err(());
        }

        // Check that both nodes have the desired ports.
        match port_type {
            PortType::Audio => {
                if source_node_port_id >= source_node.n_audio_out_ports {
                    return Err(());
                }
                if dest_node_port_id >= dest_node.n_audio_in_ports {
                    return Err(());
                }
            }
        }

        // Check if the connection already exists.
        for self_connected_to in source_node.self_connected_to.iter() {
            if self_connected_to.port_type == port_type
                && &self_connected_to.dest_node_id == dest_node_id
                && self_connected_to.source_port_id == source_node_port_id
                && self_connected_to.dest_port_id == dest_node_port_id
            {
                return Ok(());
            }
        }

        let new_port_connection = NodeConnection {
            source_node_id: source_node_id.clone(),
            dest_node_id: dest_node_id.clone(),
            source_port_id: source_node_port_id,
            dest_port_id: dest_node_port_id,
            port_type,
        };

        self.node_map
            .get_mut(source_node_id)
            .unwrap()
            .self_connected_to
            .push(new_port_connection.clone());
        self.node_map
            .get_mut(dest_node_id)
            .unwrap()
            .connected_to_self
            .push(new_port_connection);

        Ok(())
    }

    // TODO: Return custom error type.
    pub(super) fn remove_port_connection(
        &mut self,
        port_type: PortType,
        source_node_id: &String,
        source_node_port_id: usize,
        dest_node_id: &String,
        dest_node_port_id: usize,
    ) -> Result<(), ()> {
        {
            // Check that both nodes are different.
            if source_node_id == dest_node_id {
                return Err(());
            }

            let source_node = if let Some(n) = self.node_map.get_mut(source_node_id) {
                n
            } else {
                return Err(());
            };

            let mut self_port = None;
            for (self_i, self_connected_to) in source_node.self_connected_to.iter().enumerate() {
                if self_connected_to.port_type == port_type
                    && &self_connected_to.dest_node_id == dest_node_id
                    && self_connected_to.source_port_id == source_node_port_id
                    && self_connected_to.dest_port_id == dest_node_port_id
                {
                    self_port = Some(self_i);
                    break;
                }
            }
            if let Some(port_id) = self_port {
                source_node.self_connected_to.remove(port_id);
            } else {
                return Err(());
            }
        }

        let dest_node = if let Some(n) = self.node_map.get_mut(dest_node_id) {
            n
        } else {
            return Err(());
        };

        let mut dest_port = None;
        for (dest_i, connected_to_self) in dest_node.connected_to_self.iter().enumerate() {
            if connected_to_self.port_type == port_type
                && &connected_to_self.source_node_id == source_node_id
                && connected_to_self.source_port_id == source_node_port_id
                && connected_to_self.dest_port_id == dest_node_port_id
            {
                dest_port = Some(dest_i);
                break;
            }
        }
        if let Some(port_id) = dest_port {
            dest_node.connected_to_self.remove(port_id);
        } else {
            return Err(());
        }

        Ok(())
    }

    // TODO: Return custom error type.
    pub(super) fn compile(&mut self) -> Result<(), ()> {
        Ok(())
    }
}
