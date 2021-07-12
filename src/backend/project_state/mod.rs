use basedrop::Collector;
use fnv::FnvHashMap;
use fnv::FnvHashSet;
use std::path::PathBuf;

use crate::backend::pcm::{AnyPcm, MonoPcm, PcmLoadError, PcmLoader, StereoPcm};
use crate::backend::timeline::{MonoAudioClip, StereoAudioClip};

/// This struct should contain all information needed to create a "save file"
/// for the project.
///
/// All operations that affect the project state must happen through one of this struct's
/// methods. As such this struct just be responsible for checking that the project state
/// always remains valid. This will also allow us to create a scripting api later on.
///
/// TODO: Project file format. This will need to be future-proof.
pub struct ProjectState {
    /// The audio clips in this project. Each audio clip *must* have a unique ID.
    mono_audio_clips: FnvHashMap<String, MonoAudioClip>,
    stereo_audio_clips: FnvHashMap<String, StereoAudioClip>,

    pcm_loader: PcmLoader,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            mono_audio_clips: FnvHashMap::default(),
            stereo_audio_clips: FnvHashMap::default(),

            pcm_loader: PcmLoader::new(),
        }
    }
}
