pub mod stereo_gain;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum NodeConnectionType {
    StereoAudio,
    MonoAudio,
    Midi,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeConnection {
    pub output_node_id: String,
    pub output_port_id: usize,
    pub input_node_id: String,
    pub input_port_id: usize,
    pub conn_type: NodeConnectionType,
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

    fn pop_message(&mut self) -> Option<NodeMessage>;
}

pub trait AudioGraphNode: Send {
    fn process(&mut self, audio_buffers: &mut [Vec<f32>]);
}

#[derive(Debug)]
pub enum NodeMessage {
    StereoGain(stereo_gain::Message),
}
