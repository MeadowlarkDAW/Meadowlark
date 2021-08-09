use std::path::PathBuf;
use std::hash::{Hash, Hasher};

use basedrop::{Handle, Shared};
use rusty_daw_time::{SampleRate, SampleTime, Seconds, TempoMap};

use super::AudioClipSaveState;
use crate::backend::resource_loader::{AnyPcm, MonoPcm, StereoPcm, PcmLoadError, PcmLoader};
use crate::util::TwoXHashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum AudioClipResourceType {
    /// If the clip does not need any resampling (because the original sample rate
    /// is the same as the project's sample rate, and there are no pitch shifting or
    /// time stretching effects), then just use the original samples.
    Original,

    /// Used when the only change made is to resample the clip from its original
    /// sample rate to the project's sample rate, with no pitch shifting or
    /// time stretching.
    ///
    /// Since this is very common, we opt to deallocate the original samples from
    /// memory and use this instead. However, once an edit to pitch shifting or time
    /// stretching is made, then the original will be re-loaded from disk to avoid
    /// re-resampling (which has poor sound quality).
    OnlySampleRateChange,

    /// Used when the clip has pitch shifting and/or time stretching effects applied.
    ///
    /// In this case we will store the original samples in memory since the user
    /// is likely to want to edit these parameters again. This is so we can avoid
    /// re-resampling (which has poor sound quality);
    HasEffects,

    // TODO: Streamed from disk type.
    //
    // Note, due to the nature of streaming from disk, all resampling must be done
    // at playback. Because of this, time stretching effects may prove to be
    // unfeasible for streamed audio clips. We will probably end up using
    // destructive editing in that case by asking the user to render the audio
    // clip into a new file in order to apply the effect.
}

// The following is only relevant when the type is `HasEffects`. I'm not
// sure how Rust handles hashing enums, so I just put these here to make sure the
// hash always stays the same when the type is not `HasEffects`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct EffectKeyParams {
    duration: SampleTime,
    clip_start_offset: SampleTime,

    // TODO: pitch shifting and time stretching
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioClipResourceKey {
    pcm_path: PathBuf,
    rsac_type: AudioClipResourceType,

    effect_params: Option<EffectKeyParams>,
}

impl Hash for AudioClipResourceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pcm_path.hash(state);
        self.rsac_type.hash(state);

        if let Some(params) = self.effect_params {
            params.hash(state);
        }
    }
}

pub struct AudioClipResource {
    pub pcm: Shared<AnyPcm>,

    /// This is the start offset to the samples in the rendered `pcm`. This may not
    /// necessarily be the same as the start offset in the audio clip's save state.
    pub start_offset: SampleTime,

    /// When the rendered type is `HasEffects`, we want to keep the original samples
    /// around in memory since the user is likely to want to edit the pitch shifting
    /// and/or time stretching effects again.
    original: Option<Shared<AnyPcm>>,
}

pub struct AudioClipResampler {
    resampled: TwoXHashMap<AudioClipResourceKey, Shared<AudioClipResource>>,

    sample_rate: SampleRate,
    
    coll_handle: Handle,
}

impl AudioClipResampler {
    pub fn new(coll_handle: Handle, sample_rate: SampleRate) -> Self {
        Self {
            resampled: Default::default(),
            sample_rate,
            coll_handle,
        }
    }

    pub fn render(&mut self, state: &AudioClipSaveState, loader: &mut PcmLoader) -> (Shared<AudioClipResource>, Result<(), PcmLoadError>) {
        

        todo!()
    }
}