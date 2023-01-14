use audio_graph::Edge;
use fnv::FnvHashMap;

use crate::processor_schedule::tasks::{
    SharedAudioDelayCompNode, SharedAutomationDelayCompNode, SharedNoteDelayCompNode,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DelayCompKey {
    pub edge: Edge,
    pub delay: u32,
}

pub(crate) struct DelayCompNodePool {
    pub audio: FnvHashMap<DelayCompKey, SharedAudioDelayCompNode>,
    pub note: FnvHashMap<DelayCompKey, SharedNoteDelayCompNode>,
    pub automation: FnvHashMap<DelayCompKey, SharedAutomationDelayCompNode>,
}

impl DelayCompNodePool {
    pub fn new() -> Self {
        Self {
            audio: FnvHashMap::default(),
            note: FnvHashMap::default(),
            automation: FnvHashMap::default(),
        }
    }
}
