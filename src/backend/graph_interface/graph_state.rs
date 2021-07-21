use fnv::FnvHashMap;

use super::node::AudioGraphNode;

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
pub struct NodeID(u64);

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortType {
    MonoAudio,
    StereoAudio,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NodeConnection {
    pub source_node_id: NodeID,
    pub dest_node_id: NodeID,
    pub source_port_index: usize,
    pub dest_port_index: usize,
    pub port_type: PortType,
}

pub struct NodeState {
    pub n_mono_audio_in_ports: usize,
    pub n_mono_audio_out_ports: usize,
    pub n_stereo_audio_in_ports: usize,
    pub n_stereo_audio_out_ports: usize,

    pub self_connected_to: Vec<NodeConnection>,
    pub connected_to_self: Vec<NodeConnection>,

    pub(super) node_pool_index: usize,
}

pub struct GraphState {
    node_map: FnvHashMap<NodeID, NodeState>,
    next_node_id: u64,
}

impl GraphState {
    pub(super) fn new() -> Self {
        Self {
            node_map: FnvHashMap::default(),
            next_node_id: 0,
        }
    }

    pub fn get_node_state(&self, node_id: &NodeID) -> Option<&NodeState> {
        self.node_map.get(node_id)
    }

    // TODO: Return custom error type.
    pub(super) fn add_new_node(&mut self, node: &Box<dyn AudioGraphNode>) -> NodeID {
        let node_id = NodeID(self.next_node_id);
        self.next_node_id += 1;

        let n_mono_audio_in_ports = node.mono_audio_in_ports();
        let n_mono_audio_out_ports = node.mono_audio_out_ports();
        let n_stereo_audio_in_ports = node.stereo_audio_in_ports();
        let n_stereo_audio_out_ports = node.stereo_audio_out_ports();

        self.node_map.insert(
            node_id,
            NodeState {
                node_pool_index: self.node_map.len(),

                n_mono_audio_in_ports,
                n_mono_audio_out_ports,
                n_stereo_audio_in_ports,
                n_stereo_audio_out_ports,

                self_connected_to: Vec::new(),
                connected_to_self: Vec::new(),
            },
        );

        node_id
    }

    // TODO: Return custom error type.
    pub(super) fn remove_node(&mut self, node_id: &NodeID) -> Result<usize, ()> {
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

    // Replace a node with another node while attempting to keep existing connections.
    pub(super) fn replace_node(
        &mut self,
        node_id: &NodeID,
        new_node: &Box<dyn AudioGraphNode>,
    ) -> Result<usize, ()> {
        let mut prev_node_state = if let Some(node) = self.node_map.remove(node_id) {
            node
        } else {
            return Err(());
        };

        let n_mono_audio_in_ports = new_node.mono_audio_in_ports();
        let n_mono_audio_out_ports = new_node.mono_audio_out_ports();
        let n_stereo_audio_in_ports = new_node.stereo_audio_in_ports();
        let n_stereo_audio_out_ports = new_node.stereo_audio_out_ports();

        prev_node_state
            .self_connected_to
            .retain(|self_connected_to| {
                let do_retain = match self_connected_to.port_type {
                    PortType::MonoAudio => {
                        self_connected_to.source_port_index < n_mono_audio_out_ports
                    }
                    PortType::StereoAudio => {
                        self_connected_to.source_port_index < n_stereo_audio_out_ports
                    }
                };

                if !do_retain {
                    // Remove the connection from the connected node.
                    self.node_map
                        .get_mut(&self_connected_to.dest_node_id)
                        .unwrap()
                        .connected_to_self
                        .retain(|n| n != self_connected_to);
                }

                do_retain
            });
        prev_node_state
            .connected_to_self
            .retain(|connected_to_self| {
                let do_retain = match connected_to_self.port_type {
                    PortType::MonoAudio => {
                        connected_to_self.dest_port_index < n_mono_audio_in_ports
                    }
                    PortType::StereoAudio => {
                        connected_to_self.dest_port_index < n_stereo_audio_in_ports
                    }
                };

                if !do_retain {
                    // Remove the connection from the connected node.
                    self.node_map
                        .get_mut(&connected_to_self.source_node_id)
                        .unwrap()
                        .self_connected_to
                        .retain(|n| n != connected_to_self);
                }

                do_retain
            });

        let node_pool_index = prev_node_state.node_pool_index;

        self.node_map.insert(
            *node_id,
            NodeState {
                node_pool_index: prev_node_state.node_pool_index,

                n_mono_audio_in_ports,
                n_mono_audio_out_ports,
                n_stereo_audio_in_ports,
                n_stereo_audio_out_ports,

                self_connected_to: prev_node_state.self_connected_to,
                connected_to_self: prev_node_state.connected_to_self,
            },
        );

        Ok(node_pool_index)
    }

    // TODO: Return custom error type.
    pub(super) fn add_port_connection(
        &mut self,
        port_type: PortType,
        source_node_id: &NodeID,
        source_node_port_index: usize,
        dest_node_id: &NodeID,
        dest_node_port_index: usize,
    ) -> Result<(), ()> {
        // TODO: Detect cycles.

        // Check that both nodes exist.
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
            PortType::MonoAudio => {
                if source_node_port_index >= source_node.n_mono_audio_out_ports {
                    return Err(());
                }
                if dest_node_port_index >= dest_node.n_mono_audio_in_ports {
                    return Err(());
                }
            }
            PortType::StereoAudio => {
                if source_node_port_index >= source_node.n_stereo_audio_out_ports {
                    return Err(());
                }
                if dest_node_port_index >= dest_node.n_stereo_audio_in_ports {
                    return Err(());
                }
            }
        }

        // Check if the input port on the dest node is already connected to another node.
        for connected_to_self in dest_node.connected_to_self.iter() {
            if connected_to_self.port_type == port_type
                && connected_to_self.dest_port_index == dest_node_port_index
            {
                return Err(());
            }
        }

        let new_port_connection = NodeConnection {
            source_node_id: *source_node_id,
            dest_node_id: *dest_node_id,
            source_port_index: source_node_port_index,
            dest_port_index: dest_node_port_index,
            port_type,
        };

        self.node_map
            .get_mut(source_node_id)
            .unwrap()
            .self_connected_to
            .push(new_port_connection);
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
        source_node_id: &NodeID,
        source_node_port_index: usize,
        dest_node_id: &NodeID,
        dest_node_port_index: usize,
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
                    && self_connected_to.source_port_index == source_node_port_index
                    && self_connected_to.dest_port_index == dest_node_port_index
                {
                    self_port = Some(self_i);
                    break;
                }
            }
            if let Some(port_index) = self_port {
                source_node.self_connected_to.remove(port_index);
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
                && connected_to_self.source_port_index == source_node_port_index
                && connected_to_self.dest_port_index == dest_node_port_index
            {
                dest_port = Some(dest_i);
                break;
            }
        }
        if let Some(port_index) = dest_port {
            dest_node.connected_to_self.remove(port_index);
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
