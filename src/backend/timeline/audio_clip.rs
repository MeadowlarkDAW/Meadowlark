use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_time::{MusicalTime, SampleRate, SampleTime, Seconds, TempoMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::backend::generic_nodes::{DB_GRADIENT, SMOOTH_SECS};
use crate::backend::graph_interface::{ProcInfo, StereoAudioBlockBuffer};
use crate::backend::parameter::{ParamF32, ParamF32Handle, Unit};
use crate::backend::resource_loader::{pcm, AnyPcm, PcmLoadError, ResourceLoader};
use crate::backend::timeline::TimelineTransport;
use crate::backend::MAX_BLOCKSIZE;

use super::sampler::sample_stereo;

pub static AUDIO_CLIP_GAIN_MIN_DB: f32 = -40.0;
pub static AUDIO_CLIP_GAIN_MAX_DB: f32 = 40.0;

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
        save_state: &mut AudioClipSaveState,
    ) {
        save_state.clip_start_offset = clip_start_offset;

        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.clip_start_offset = clip_start_offset;

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    /// Set the PCM resource to use from the given path to an audio file.
    pub fn set_pcm(
        &mut self,
        pcm_path: PathBuf,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        save_state: &mut AudioClipSaveState,
    ) -> Result<(), PcmLoadError> {
        let (pcm, res) = { resource_loader.lock().unwrap().pcm_loader.load(&pcm_path) };

        save_state.pcm_path = pcm_path;

        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.pcm = pcm;

        self.info.set(Shared::new(&self.coll_handle, new_info));

        res
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
    // PcmResources are always immutable. This reflects the non-destructive nature
    // of this sampler engine.
    pub pcm: Shared<AnyPcm>,

    pub timeline_start: SampleTime,
    pub timeline_end: SampleTime,

    pub clip_start_offset: Seconds,
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
            coll_handle.clone(),
        );

        let (pcm, res) = {
            resource_loader
                .lock()
                .unwrap()
                .pcm_loader
                .load(&save_state.pcm_path)
        };

        let timeline_start = tempo_map.musical_to_nearest_sample_round(save_state.timeline_start);
        let timeline_end = tempo_map.seconds_to_nearest_sample_round(
            save_state.timeline_start.to_seconds(&tempo_map) + save_state.duration,
        );

        let info = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                AudioClipProcInfo {
                    pcm,
                    timeline_start,
                    timeline_end,
                    clip_start_offset: save_state.clip_start_offset,
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
            res,
        )
    }

    pub fn process(
        &self,
        playhead: SampleTime,
        frames: usize,
        sample_rate: SampleRate,
        out: &mut StereoAudioBlockBuffer,
        out_offset: usize,
    ) {
        let info = self.info.get();

        // Find the time in seconds to start reading from in the PCM resource.
        let pcm_start =
            (playhead - info.timeline_start).to_seconds(sample_rate) + info.clip_start_offset;

        let mut params = self.params.borrow_mut();
        let amp = params.clip_gain_amp.smoothed(frames);

        match &*info.pcm {
            AnyPcm::Mono(pcm) => {}
            AnyPcm::Stereo(pcm) => {
                sample_stereo(frames, sample_rate, pcm, pcm_start, out, out_offset)
            }
        }

        let apply_amp = if amp.is_smoothing() {
            true
        } else {
            // Don't need to apply gain if amp is 1.0.
            amp[0] != 1.0
        };
        if apply_amp {
            // Tell compiler we want to optimize loops. (The min() condition should never actually happen.)
            let frames = frames.min(MAX_BLOCKSIZE);
            let out_offset = out_offset.min(MAX_BLOCKSIZE - frames);

            for i in 0..frames {
                out.left[out_offset + i] *= amp[i];
                out.right[out_offset + i] *= amp[i];
            }
        }
    }

    /// Clear any buffers.
    pub fn clear(&mut self) {
        // Nothing to clear at the moment.
    }
}
