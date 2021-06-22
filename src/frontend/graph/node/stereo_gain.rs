use super::{AudioGraphNode, AudioGraphNodeState, NodeConnection, NodeConnectionType, NodeMessage};

#[derive(Debug)]
pub struct State {
    connection: Option<NodeConnection>,
    amp: f32,

    connections_changed: bool,

    // Only one message is needed for this simple node
    node_message: Option<NodeMessage>,
}

impl State {
    pub fn new(connection: Option<NodeConnection>, amp: f32) -> Self {
        Self {
            connection,
            amp,
            connections_changed: false,
            node_message: None,
        }
    }

    pub fn connect_to(&mut self, node_id: String, stereo_out_port_id: usize) {
        self.connection = Some(NodeConnection {
            node_id,
            port_id: stereo_out_port_id,
            conn_type: NodeConnectionType::StereoAudio,
        });
        self.connections_changed = true;
    }

    pub fn set_amp(&mut self, amp: f32) {
        if self.amp != amp {
            self.amp = amp;
            self.node_message = Some(NodeMessage::StereoGain(Message::SetAmp(amp)));
        }
    }

    pub fn amp(&self) -> f32 {
        self.amp
    }

    pub fn connection(&self) -> &Option<NodeConnection> {
        &self.connection
    }
}

impl AudioGraphNodeState for State {
    fn num_stereo_outputs(&self) -> usize {
        1
    }

    fn connections(&self) -> &[NodeConnection] {
        if let Some(conn) = self.connection {
            &[conn]
        } else {
            &[]
        }
    }

    fn parameters_changed(&self) -> bool {
        self.node_message.is_some()
    }
    fn connections_changed(&mut self) -> bool {
        if self.connections_changed {
            self.connections_changed = false;
            true
        } else {
            false
        }
    }
    fn pop_message(&mut self) -> Option<NodeMessage> {
        self.node_message.take()
    }
}

pub struct Node {
    pub buffer_index: usize,
    pub amp: f32,
}

impl Node {}

impl AudioGraphNode for Node {
    fn process(&mut self, audio_buffers: &mut [Vec<f32>]) {}
}

#[derive(Debug)]
pub enum Message {
    SetAmp(f32),
}
