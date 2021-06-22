pub mod stereo_gain;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeConnectionType {
    StereoAudio,
    MonoAudio,
    Midi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeConnection {
    node_id: String,
    port_id: usize,
    conn_type: NodeConnectionType,
}

impl NodeConnection {
    pub fn node_id(&self) -> &String {
        &self.node_id
    }

    pub fn port_id(&self) -> usize {
        self.port_id
    }

    pub fn conn_type(&self) -> &NodeConnectionType {
        &self.conn_type
    }
}

pub trait AudioGraphNodeState {
    fn num_stereo_outputs(&self) -> usize {
        0
    }
    fn num_mono_output(&self) -> usize {
        0
    }
    fn num_midi_outputs(&self) -> usize {
        0
    }

    fn connections(&self) -> &[NodeConnection];

    fn parameters_changed(&self) -> bool;
    fn connections_changed(&mut self) -> bool;
    fn pop_message(&mut self) -> Option<NodeMessage>;
}

pub trait AudioGraphNode: Send {
    fn process(&mut self, audio_buffers: &mut [Vec<f32>]);
}

#[derive(Debug)]
pub enum NodeMessage {
    StereoGain(stereo_gain::Message),
}
