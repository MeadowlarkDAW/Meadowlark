mod graph;

pub use graph::{AudioGraph, AudioGraphState};

use graph::AudioGraphMsg;

pub struct FrontendState {
    pub audio_graph_state: AudioGraphState,
}

impl FrontendState {
    pub fn new() -> (Self, BackendState) {
        let (audio_graph_state, audio_graph) = AudioGraphState::new();

        (Self { audio_graph_state }, BackendState { audio_graph })
    }

    // TODO: have some way to periodically call `audio_graph_state.collect()` to collect
    // any garbage.
}

pub struct BackendState {
    pub audio_graph: AudioGraph,
}

impl BackendState {
    pub fn sync(&mut self) {
        self.audio_graph.sync();
    }
}

#[derive(Debug, Clone)]
enum StateToRtMsg {
    AudioGraph(AudioGraphMsg),
}
