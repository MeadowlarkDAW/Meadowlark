use basedrop::Collector;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::path::PathBuf;

use crate::backend::pcm::{AnyPcm, MonoPcm, PcmLoadError, PcmLoader, StereoPcm};

/// This struct should contain all information needed to create a "save file"
/// for the project.
///
/// All operations that affect the project state must happen through one of this struct's
/// methods. As such this struct just be responsible for checking that the project state
/// always remains valid. This will also allow us to create a scripting api later on.
///
/// TODO: Project file format. This will need to be future-proof.
pub struct ProjectState {
    pcm_loader: PcmLoader,

    collector: Collector,
}

impl ProjectState {
    pub fn new() -> Self {
        let collector = Collector::new();

        Self {
            pcm_loader: PcmLoader::new(collector.handle()),

            collector,
        }
    }
}
