use audio_graph::{NodeRef, PortRef};

#[non_exhaustive]
#[repr(u8)] // pad `PortIdent` to be 32bit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortType {
    MonoAudio,
    StereoAudio,
    // TODO: Control
}

impl Default for PortType {
    fn default() -> Self {
        PortType::MonoAudio
    }
}

impl audio_graph::PortType for PortType {
    const NUM_TYPES: usize = 2;

    #[inline]
    fn id(&self) -> usize {
        match self {
            PortType::MonoAudio => 0,
            PortType::StereoAudio => 1,
        }
    }
}

#[repr(u8)] // pad `PortIdent` to be 32bit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortPlacement {
    Input,
    Output,
}

impl Default for PortPlacement {
    fn default() -> Self {
        PortPlacement::Input
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PortIdent {
    pub port_type: PortType,
    pub placement: PortPlacement,
    pub index: u16,
}

#[derive(Default)]
pub struct NodeState {
    mono_audio_in_port_refs: Vec<PortRef>,
    mono_audio_out_port_refs: Vec<PortRef>,
    stereo_audio_in_port_refs: Vec<PortRef>,
    stereo_audio_out_port_refs: Vec<PortRef>,
}

impl NodeState {
    #[inline]
    pub fn mono_audio_in_ports(&self) -> usize {
        self.mono_audio_in_port_refs.len()
    }

    #[inline]
    pub fn mono_audio_out_ports(&self) -> usize {
        self.mono_audio_out_port_refs.len()
    }

    #[inline]
    pub fn stereo_audio_in_ports(&self) -> usize {
        self.stereo_audio_in_port_refs.len()
    }

    #[inline]
    pub fn stereo_audio_out_ports(&self) -> usize {
        self.stereo_audio_out_port_refs.len()
    }
}

pub(super) struct GraphState {
    pub node_states: Vec<NodeState>,
    pub graph: audio_graph::Graph<NodeRef, PortIdent, PortType>,
}

impl GraphState {
    pub fn new() -> Self {
        Self { node_states: Vec::new(), graph: audio_graph::Graph::default() }
    }

    pub fn get_node_state(&self, node_ref: NodeRef) -> Result<&NodeState, audio_graph::Error> {
        self.graph.node_check(node_ref).map(|_| {
            let index: usize = node_ref.into();
            &self.node_states[index]
        })
    }

    pub fn set_num_ports(
        &mut self,
        node_ref: NodeRef,
        mono_audio_in_ports: u32,
        mono_audio_out_ports: u32,
        stereo_audio_in_ports: u32,
        stereo_audio_out_ports: u32,
    ) -> Result<(), audio_graph::Error> {
        self.graph.node_check(node_ref)?;

        let node_index: usize = node_ref.into();
        let node_state = &mut self.node_states[node_index];

        while node_state.mono_audio_in_port_refs.len() < mono_audio_in_ports as usize {
            let port_ref = self
                .graph
                .port(
                    node_ref,
                    PortType::MonoAudio,
                    PortIdent {
                        port_type: PortType::MonoAudio,
                        placement: PortPlacement::Input,
                        index: node_state.mono_audio_in_port_refs.len() as u16,
                    },
                )
                .unwrap();
            node_state.mono_audio_in_port_refs.push(port_ref);
        }
        while node_state.mono_audio_in_port_refs.len() > mono_audio_in_ports as usize {
            let last_port_ref = node_state.mono_audio_in_port_refs.pop().unwrap();
            self.graph.delete_port(last_port_ref).unwrap();
        }

        while node_state.mono_audio_out_port_refs.len() < mono_audio_out_ports as usize {
            let port_ref = self
                .graph
                .port(
                    node_ref,
                    PortType::MonoAudio,
                    PortIdent {
                        port_type: PortType::MonoAudio,
                        placement: PortPlacement::Output,
                        index: node_state.mono_audio_out_port_refs.len() as u16,
                    },
                )
                .unwrap();
            node_state.mono_audio_out_port_refs.push(port_ref);
        }
        while node_state.mono_audio_out_port_refs.len() > mono_audio_out_ports as usize {
            let last_port_ref = node_state.mono_audio_out_port_refs.pop().unwrap();
            self.graph.delete_port(last_port_ref).unwrap();
        }

        while node_state.stereo_audio_in_port_refs.len() < stereo_audio_in_ports as usize {
            let port_ref = self
                .graph
                .port(
                    node_ref,
                    PortType::StereoAudio,
                    PortIdent {
                        port_type: PortType::StereoAudio,
                        placement: PortPlacement::Input,
                        index: node_state.stereo_audio_in_port_refs.len() as u16,
                    },
                )
                .unwrap();
            node_state.stereo_audio_in_port_refs.push(port_ref);
        }
        while node_state.stereo_audio_in_port_refs.len() > stereo_audio_in_ports as usize {
            let last_port_ref = node_state.stereo_audio_in_port_refs.pop().unwrap();
            self.graph.delete_port(last_port_ref).unwrap();
        }

        while node_state.stereo_audio_out_port_refs.len() < stereo_audio_out_ports as usize {
            let port_ref = self
                .graph
                .port(
                    node_ref,
                    PortType::StereoAudio,
                    PortIdent {
                        port_type: PortType::StereoAudio,
                        placement: PortPlacement::Output,
                        index: node_state.stereo_audio_out_port_refs.len() as u16,
                    },
                )
                .unwrap();
            node_state.stereo_audio_out_port_refs.push(port_ref);
        }
        while node_state.stereo_audio_out_port_refs.len() > stereo_audio_out_ports as usize {
            let last_port_ref = node_state.stereo_audio_out_port_refs.pop().unwrap();
            self.graph.delete_port(last_port_ref).unwrap();
        }

        Ok(())
    }

    pub fn add_new_node(
        &mut self,
        mono_audio_in_ports: u32,
        mono_audio_out_ports: u32,
        stereo_audio_in_ports: u32,
        stereo_audio_out_ports: u32,
    ) -> NodeRef {
        let node_ref = self.graph.node(NodeRef::default());
        // We're using the node reference as the identifier. The node reference is just an index
        // into a Vec. We *could* use our own identifier system with a hashmap, but since the
        // `audio-graph` crate is already handling this kind of logic, we might as well use it
        // for our nodes too.
        self.graph.set_node_ident(node_ref, node_ref).unwrap();

        let index: usize = node_ref.into();
        while index >= self.node_states.len() {
            self.node_states.push(NodeState::default());
        }

        let mono_audio_in_port_refs = (0..mono_audio_in_ports as u16)
            .map(|i| {
                self.graph
                    .port(
                        node_ref,
                        PortType::MonoAudio,
                        PortIdent {
                            port_type: PortType::MonoAudio,
                            placement: PortPlacement::Input,
                            index: i,
                        },
                    )
                    .unwrap()
            })
            .collect();
        let mono_audio_out_port_refs = (0..mono_audio_out_ports as u16)
            .map(|i| {
                self.graph
                    .port(
                        node_ref,
                        PortType::MonoAudio,
                        PortIdent {
                            port_type: PortType::MonoAudio,
                            placement: PortPlacement::Output,
                            index: i,
                        },
                    )
                    .unwrap()
            })
            .collect();
        let stereo_audio_in_port_refs = (0..stereo_audio_in_ports as u16)
            .map(|i| {
                self.graph
                    .port(
                        node_ref,
                        PortType::MonoAudio,
                        PortIdent {
                            port_type: PortType::StereoAudio,
                            placement: PortPlacement::Input,
                            index: i,
                        },
                    )
                    .unwrap()
            })
            .collect();
        let stereo_audio_out_port_refs = (0..stereo_audio_out_ports as u16)
            .map(|i| {
                self.graph
                    .port(
                        node_ref,
                        PortType::MonoAudio,
                        PortIdent {
                            port_type: PortType::StereoAudio,
                            placement: PortPlacement::Output,
                            index: i,
                        },
                    )
                    .unwrap()
            })
            .collect();

        self.node_states[index] = NodeState {
            mono_audio_in_port_refs,
            mono_audio_out_port_refs,
            stereo_audio_in_port_refs,
            stereo_audio_out_port_refs,
        };

        node_ref
    }

    pub fn remove_node(&mut self, node_ref: NodeRef) -> Result<(), audio_graph::Error> {
        self.graph.delete_node(node_ref)?;

        Ok(())
    }

    pub fn connect_ports(
        &mut self,
        port_type: PortType,
        source_node_ref: NodeRef,
        source_node_port_index: usize,
        dest_node_ref: NodeRef,
        dest_node_port_index: usize,
    ) -> Result<(), audio_graph::Error> {
        // Check that both ports exist.
        let src_node_index: usize = source_node_ref.into();
        let dest_node_index: usize = dest_node_ref.into();
        let (src_port_ref, dest_port_ref) = match port_type {
            PortType::MonoAudio => {
                if source_node_port_index >= self.node_states[src_node_index].mono_audio_out_ports()
                    || dest_node_port_index
                        >= self.node_states[dest_node_index].mono_audio_in_ports()
                {
                    return Err(audio_graph::Error::PortDoesNotExist);
                } else {
                    (
                        self.node_states[src_node_index].mono_audio_out_port_refs
                            [source_node_port_index],
                        self.node_states[dest_node_index].mono_audio_in_port_refs
                            [dest_node_port_index],
                    )
                }
            }
            PortType::StereoAudio => {
                if source_node_port_index
                    >= self.node_states[src_node_index].stereo_audio_out_ports()
                    || dest_node_port_index
                        >= self.node_states[dest_node_index].stereo_audio_in_ports()
                {
                    return Err(audio_graph::Error::PortDoesNotExist);
                } else {
                    (
                        self.node_states[src_node_index].stereo_audio_out_port_refs
                            [source_node_port_index],
                        self.node_states[dest_node_index].stereo_audio_in_port_refs
                            [dest_node_port_index],
                    )
                }
            }
        };

        // Check that both nodes are different.
        if source_node_ref == dest_node_ref {
            return Err(audio_graph::Error::Cycle);
        }

        // Connect the two ports in the graph. This should also return an error if the destination
        // port was already connected to another port, or if a cycle was detected.
        self.graph.connect(src_port_ref, dest_port_ref)?;

        Ok(())
    }

    pub fn disconnect_ports(
        &mut self,
        port_type: PortType,
        source_node_ref: NodeRef,
        source_node_port_index: usize,
        dest_node_ref: NodeRef,
        dest_node_port_index: usize,
    ) -> Result<(), audio_graph::Error> {
        // Check that both ports exist.
        let src_node_index: usize = source_node_ref.into();
        let dest_node_index: usize = dest_node_ref.into();
        let (src_port_ref, dest_port_ref) = match port_type {
            PortType::MonoAudio => {
                if source_node_port_index >= self.node_states[src_node_index].mono_audio_out_ports()
                    || dest_node_port_index
                        >= self.node_states[dest_node_index].mono_audio_in_ports()
                {
                    return Err(audio_graph::Error::PortDoesNotExist);
                } else {
                    (
                        self.node_states[src_node_index].mono_audio_out_port_refs
                            [source_node_port_index],
                        self.node_states[dest_node_index].mono_audio_in_port_refs
                            [dest_node_port_index],
                    )
                }
            }
            PortType::StereoAudio => {
                if source_node_port_index
                    >= self.node_states[src_node_index].stereo_audio_out_ports()
                    || dest_node_port_index
                        >= self.node_states[dest_node_index].stereo_audio_in_ports()
                {
                    return Err(audio_graph::Error::PortDoesNotExist);
                } else {
                    (
                        self.node_states[src_node_index].stereo_audio_out_port_refs
                            [source_node_port_index],
                        self.node_states[dest_node_index].stereo_audio_in_port_refs
                            [dest_node_port_index],
                    )
                }
            }
        };

        self.graph.disconnect(src_port_ref, dest_port_ref)?;

        Ok(())
    }
}
