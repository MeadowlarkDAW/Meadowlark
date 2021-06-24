use super::{AudioGraphNode, AudioGraphNodeState, NodeMessage};

#[derive(Debug)]
pub struct State {
    amp: f32,

    // Only one message is needed for this simple node
    node_message: Option<NodeMessage>,
}

impl State {
    pub fn new(amp: f32) -> Self {
        Self {
            amp,
            node_message: None,
        }
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
}

impl AudioGraphNodeState for State {
    fn num_stereo_outputs(&self) -> usize {
        1
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
