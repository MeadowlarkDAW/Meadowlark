use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_time::{MusicalTime, SampleRate, SampleTime, Seconds, TempoMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::backend::audio_graph::StereoAudioBlockBuffer;
use crate::backend::generic_nodes::{DB_GRADIENT, SMOOTH_SECS};
use crate::backend::parameter::{ParamF32, ParamF32Handle, Unit};
use crate::backend::resource_loader::{AnyPcm, PcmLoadError, ResourceLoader, StereoPcm};
use crate::backend::MAX_BLOCKSIZE;

pub static AUDIO_CLIP_GAIN_MIN_DB: f32 = -40.0;
pub static AUDIO_CLIP_GAIN_MAX_DB: f32 = 40.0;

mod resource;
pub use resource::{AudioClipResource, AudioClipResourceCache};

#[derive(Debug, Clone)]
pub struct AudioClipSaveState {
    /// The ID (name) of the audio clip. This must be unique for
    /// each audio clip.
    pub id: String,

    /// The path to the audio file containing the PCM data.
    pub pcm_path: PathBuf,

    /// Where the clip starts on the timeline.
    pub timeline_start: MusicalTime,
    /// The duration of the clip on the timeline.
    pub duration: Seconds,

    /// the offset where the clip should start playing from.
    pub clip_start_offset: Seconds,

    /// The gain of the audio clip.
    pub clip_gain_db: f32,
}

pub struct AudioClipHandle {
    clip_gain_db: ParamF32Handle,

    info: Shared<SharedCell<AudioClipProcInfo>>,
    coll_handle: Handle,
}

impl AudioClipHandle {
    /// Set the gain of this audio clip.
    ///
    /// Returns the gain (this may be clamped to fit within range of the gain parameter).
    pub fn set_clip_gain_db(&mut self, gain_db: f32, save_state: &mut AudioClipSaveState) -> f32 {
        self.clip_gain_db.set_value(gain_db);

        // Make sure value is clamped within range.
        let gain_db = self.clip_gain_db.value();
        save_state.clip_gain_db = gain_db;

        gain_db
    }

    /// Set where the clip starts on the timeline.
    pub fn set_timeline_start(
        &mut self,
        timeline_start: MusicalTime,
        tempo_map: &TempoMap,
        save_state: &mut AudioClipSaveState,
    ) {
        save_state.timeline_start = timeline_start;

        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.timeline_start =
            tempo_map.musical_to_nearest_sample_round(save_state.timeline_start);
        new_info.timeline_end = tempo_map.seconds_to_nearest_sample_round(
            save_state.timeline_start.to_seconds(tempo_map) + save_state.duration,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    /// Set the duration of the clip on the timeline.
    pub fn set_duration(
        &mut self,
        duration: Seconds,
        tempo_map: &TempoMap,
        save_state: &mut AudioClipSaveState,
    ) {
        save_state.duration = duration;

        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.timeline_end = tempo_map.seconds_to_nearest_sample_round(
            save_state.timeline_start.to_seconds(tempo_map) + save_state.duration,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    /// Set the offset where the clip should start playing from.
    pub fn set_clip_start_offset(
        &mut self,
        clip_start_offset: Seconds,
        tempo_map: &TempoMap,
        save_state: &mut AudioClipSaveState,
    ) {
        save_state.clip_start_offset = clip_start_offset;

        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.clip_start_offset =
            clip_start_offset.to_nearest_sample_round(tempo_map.sample_rate);

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    /// Set the PCM resource to use from the given path to an audio file.
    pub fn set_pcm(
        &mut self,
        pcm_path: PathBuf,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        cache: &Arc<Mutex<AudioClipResourceCache>>,
        save_state: &mut AudioClipSaveState,
    ) -> Result<(), PcmLoadError> {
        let (resource, pcm_load_res) = { cache.lock().unwrap().cache(save_state, resource_loader) };

        save_state.pcm_path = pcm_path;

        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.resource = resource;

        self.info.set(Shared::new(&self.coll_handle, new_info));

        pcm_load_res
    }

    pub(super) fn update_tempo_map(
        &mut self,
        tempo_map: &TempoMap,
        save_state: &AudioClipSaveState,
    ) {
        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.timeline_start =
            tempo_map.musical_to_nearest_sample_round(save_state.timeline_start);
        new_info.timeline_end = tempo_map.seconds_to_nearest_sample_round(
            save_state.timeline_start.to_seconds(tempo_map) + save_state.duration,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }
}

struct AudioClipParams {
    pub clip_gain_amp: ParamF32,
}

#[derive(Clone)]
pub struct AudioClipProcInfo {
    // Audio clip resources are always immutable. This reflects the non-destructive nature
    // of this sampler engine.
    pub resource: Shared<AudioClipResource>,

    pub timeline_start: SampleTime,
    pub timeline_end: SampleTime,

    pub clip_start_offset: SampleTime,
}

#[derive(Clone)]
pub struct AudioClipProcess {
    // Wrapping params in a shared pointer so we can clone this struct when compiling
    // a new list of processes. This should never cause a panic because this struct is the
    // only place this is ever borrowed.
    params: Shared<AtomicRefCell<AudioClipParams>>,

    pub(super) info: Shared<SharedCell<AudioClipProcInfo>>,
}

impl AudioClipProcess {
    pub fn new(
        save_state: &AudioClipSaveState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        cache: &Arc<Mutex<AudioClipResourceCache>>,
        tempo_map: &TempoMap,
        coll_handle: Handle,
    ) -> (Self, AudioClipHandle, Result<(), PcmLoadError>) {
        let clip_gain_db = save_state
            .clip_gain_db
            .clamp(AUDIO_CLIP_GAIN_MIN_DB, AUDIO_CLIP_GAIN_MAX_DB);

        let (gain_amp, gain_handle) = ParamF32::from_value(
            clip_gain_db,
            AUDIO_CLIP_GAIN_MIN_DB,
            AUDIO_CLIP_GAIN_MAX_DB,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_SECS,
            tempo_map.sample_rate,
        );

        let (resource, pcm_load_res) = { cache.lock().unwrap().cache(save_state, resource_loader) };

        let timeline_start = tempo_map.musical_to_nearest_sample_round(save_state.timeline_start);
        let timeline_end = tempo_map.seconds_to_nearest_sample_round(
            save_state.timeline_start.to_seconds(&tempo_map) + save_state.duration,
        );

        let info = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                AudioClipProcInfo {
                    resource,
                    timeline_start,
                    timeline_end,
                    clip_start_offset: save_state
                        .clip_start_offset
                        .to_nearest_sample_round(tempo_map.sample_rate),
                },
            )),
        );
        (
            Self {
                params: Shared::new(
                    &coll_handle,
                    AtomicRefCell::new(AudioClipParams {
                        clip_gain_amp: gain_amp,
                    }),
                ),
                info: Shared::clone(&info),
            },
            AudioClipHandle {
                clip_gain_db: gain_handle,
                info,
                coll_handle,
            },
            pcm_load_res,
        )
    }

    pub fn process(
        &self,
        playhead: SampleTime,
        frames: usize,
        out: &mut StereoAudioBlockBuffer,
        out_offset: usize,
    ) {
        let info = self.info.get();

        let mut params = self.params.borrow_mut();
        let amp = params.clip_gain_amp.smoothed(frames);

        let mut copy_frames = frames;
        let mut copy_out_offset = out_offset;
        let mut skip = 0;

        // Find the sample to start reading from in the PCM resource.
        let pcm_start =
            playhead - info.timeline_start + info.clip_start_offset - info.resource.original_offset;

        if pcm_start >= SampleTime::from_usize(info.resource.pcm.len()) {
            // Out of range. Do nothing (add silence).
            return;
        }

        let pcm_start = if pcm_start.0 < 0 {
            if pcm_start + SampleTime::from_usize(frames) <= SampleTime::new(0) {
                // Out of range. Do nothing (add silence).
                return;
            }

            // Skip frames (insert silence) until there is data.
            skip = (0 - pcm_start.0) as usize;
            copy_frames -= skip;
            copy_out_offset += skip;

            0
        } else {
            pcm_start.0 as usize
        };

        if pcm_start + copy_frames > info.resource.pcm.len() {
            // Skip frames (add silence) after the end of the resource.
            copy_frames = info.resource.pcm.len() - pcm_start;
        }

        // TODO: Audio clip fades.
        let do_apply_amp = if amp.is_smoothing() {
            true
        } else {
            // Don't need to apply gain if amp is 1.0.
            amp[0] != 1.0
        };

        // Apply gain to the samples and add them to the output.
        //
        // TODO: SIMD optimizations.
        let out_left = &mut out.left[copy_out_offset..copy_out_offset + copy_frames];
        let out_right = &mut out.right[copy_out_offset..copy_out_offset + copy_frames];
        match &*info.resource.pcm {
            AnyPcm::Mono(pcm) => {
                let src = &pcm.data()[pcm_start..pcm_start + copy_frames];

                if do_apply_amp {
                    for i in 0..copy_frames {
                        let amp = &amp.values[skip..skip + copy_frames];

                        out_left[i] += src[i] * amp[i];
                        out_right[i] += src[i] * amp[i];
                    }
                } else {
                    for i in 0..copy_frames {
                        out_left[i] += src[i];
                        out_right[i] += src[i];
                    }
                }
            }
            AnyPcm::Stereo(pcm) => {
                let src_left = &pcm.left()[pcm_start..pcm_start + copy_frames];
                let src_right = &pcm.left()[pcm_start..pcm_start + copy_frames];

                if do_apply_amp {
                    for i in 0..copy_frames {
                        let amp = &amp.values[skip..skip + copy_frames];

                        out_left[i] += src_left[i] * amp[i];
                        out_right[i] += src_right[i] * amp[i];
                    }
                } else {
                    for i in 0..copy_frames {
                        out_left[i] += src_left[i];
                        out_right[i] += src_right[i];
                    }
                }
            }
        }
    }

    /// Clear any buffers.
    pub fn clear(&mut self) {
        // Nothing to clear at the moment.
    }
}
