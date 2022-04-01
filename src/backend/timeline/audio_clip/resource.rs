use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use basedrop::{Handle, Shared};
use rusty_daw_core::{Frames, SampleRate, SuperFrames};

use super::AudioClipState;
use crate::backend::dsp::resample;
use crate::backend::resource_loader::{AnyPcm, MonoPcm, PcmLoadError, ResourceLoader, StereoPcm};
use crate::util::TwoXHashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ResampledType {
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
    duration: SuperFrames,
    clip_start_offset: SuperFrames,
    // TODO: pitch shifting and time stretching
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceKey {
    pcm_path: PathBuf,
    resampled_type: ResampledType,

    effect_params: Option<EffectKeyParams>,
}

impl Hash for ResourceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pcm_path.hash(state);
        self.resampled_type.hash(state);

        if let Some(params) = self.effect_params {
            params.hash(state);
        }
    }
}

pub struct AudioClipResource {
    /// The raw PCM resource. Note that this will always have "offline effects"
    /// including resampling to the project's sample-rate already applied.
    pub pcm: Shared<AnyPcm>,

    /// This is the start offset from the start of the original resource. This is
    /// so we can save on memory if we have multiple audio clips that reference
    /// different portions of the same PCM data.
    pub original_offset: Frames,

    pub resampled_type: ResampledType,

    /// When the rendered type is `HasEffects`, we want to keep the original samples
    /// around in memory since the user is likely to want to edit the pitch shifting
    /// and/or time stretching effects again.
    _original: Option<Shared<AnyPcm>>,
}

pub struct AudioClipResourceCache {
    resources: TwoXHashMap<ResourceKey, Shared<AudioClipResource>>,

    sample_rate: SampleRate,

    coll_handle: Handle,
}

impl AudioClipResourceCache {
    pub fn new(coll_handle: Handle, sample_rate: SampleRate) -> Self {
        Self { resources: Default::default(), sample_rate, coll_handle }
    }

    pub fn cache(
        &mut self,
        state: &AudioClipState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
    ) -> (Shared<AudioClipResource>, Result<(), PcmLoadError>) {
        // Load the resource from disk / retrieve from cache.
        let (pcm, pcm_load_res) =
            { resource_loader.lock().unwrap().pcm_loader.load(&state.pcm_path) };

        // TODO: Check for pitch shifting and time stretching effects.
        let (resampled_type, effect_params) = if pcm.sample_rate() == self.sample_rate {
            (ResampledType::Original, None)
        } else {
            (ResampledType::OnlySampleRateChange, None)
        };

        if let Some(resource) = self.resources.get(&ResourceKey {
            // TODO: Find a way to do this without cloning the path every time.
            pcm_path: state.pcm_path.clone(),
            resampled_type,
            effect_params,
        }) {
            (Shared::clone(resource), pcm_load_res)
        } else {
            // Render a new resource.

            let new_resource = Shared::new(
                &self.coll_handle,
                match resampled_type {
                    ResampledType::Original => AudioClipResource {
                        pcm,
                        original_offset: Frames::default(),
                        resampled_type,
                        _original: None,
                    },
                    ResampledType::OnlySampleRateChange => {
                        let resample_ratio = self.sample_rate.0 / pcm.sample_rate().0;

                        // TODO: Use something better than linear resampling.

                        let resampled_pcm = Shared::new(
                            &self.coll_handle,
                            match &*pcm {
                                AnyPcm::Mono(pcm) => {
                                    let res = resample::linear_resample_non_rt_mono(
                                        pcm.data(),
                                        resample_ratio,
                                    );

                                    AnyPcm::Mono(MonoPcm::new(res, self.sample_rate))
                                }
                                AnyPcm::Stereo(pcm) => {
                                    let (res_l, res_r) = resample::linear_resample_non_rt_stereo(
                                        pcm.left(),
                                        pcm.right(),
                                        resample_ratio,
                                    );

                                    AnyPcm::Stereo(StereoPcm::new(res_l, res_r, self.sample_rate))
                                }
                            },
                        );

                        AudioClipResource {
                            pcm: resampled_pcm,
                            original_offset: Frames::default(),
                            resampled_type,
                            _original: None,
                        }
                    }
                    ResampledType::HasEffects => {
                        // TODO: Pitch shifting and time stretching effects.

                        AudioClipResource {
                            pcm,
                            original_offset: Frames::default(),
                            resampled_type,
                            _original: None,
                        }
                    }
                },
            );

            let new_key =
                ResourceKey { pcm_path: state.pcm_path.clone(), resampled_type, effect_params };

            let _ = self.resources.insert(new_key, Shared::clone(&new_resource));

            (new_resource, pcm_load_res)
        }
    }

    /// Drop all audio clip resources not being currently used.
    pub fn collect(&mut self) {
        // If no other extant Shared pointers to the resource exists, then
        // remove that entry.
        self.resources.retain(|_, r| Shared::get_mut(r).is_none());
    }
}
