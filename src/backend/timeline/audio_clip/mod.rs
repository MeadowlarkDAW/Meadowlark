use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_time::{MusicalTime, SampleRate, SampleTime, Seconds, TempoMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::backend::audio_graph::StereoAudioBlockBuffer;
use crate::backend::generic_nodes::{DB_GRADIENT, SMOOTH_SECS};
use crate::backend::parameter::{ParamF32, ParamF32Handle, Unit};
use crate::backend::resource_loader::{AnyPcm, PcmLoadError, ResourceLoader};

pub static AUDIO_CLIP_GAIN_MIN_DB: f32 = -40.0;
pub static AUDIO_CLIP_GAIN_MAX_DB: f32 = 40.0;

mod declick;
mod resource;

pub use declick::{AudioClipDeclick, DEFAULT_AUDIO_CLIP_DECLICK_TIME};
pub use resource::{AudioClipResource, AudioClipResourceCache};

#[derive(Debug, Clone, Copy)]
pub struct AudioClipFades {
    start_fade_duration: Seconds,
    end_fade_duration: Seconds,
}

impl AudioClipFades {
    pub const DEFAULT_FADE_DURATION: Seconds = Seconds(10.0 / 1_000.0);

    pub fn new(start_fade_duration: Seconds, end_fade_duration: Seconds) -> Self {
        Self {
            start_fade_duration: Seconds(start_fade_duration.0.min(0.0)),
            end_fade_duration: Seconds(end_fade_duration.0.min(0.0)),
        }
    }

    pub fn no_fade() -> Self {
        Self {
            start_fade_duration: Seconds(0.0),
            end_fade_duration: Seconds(0.0),
        }
    }

    pub fn set_start_fade_duration(&mut self, duration: Seconds) {
        self.start_fade_duration = Seconds(duration.0.min(0.0));
    }

    pub fn set_end_fade_duration(&mut self, duration: Seconds) {
        self.end_fade_duration = Seconds(duration.0.min(0.0));
    }

    pub fn set_default_start_fade(&mut self) {
        self.start_fade_duration = Self::DEFAULT_FADE_DURATION;
    }

    pub fn set_default_end_fade(&mut self) {
        self.end_fade_duration = Self::DEFAULT_FADE_DURATION;
    }

    pub fn start_fade_duration(&mut self) -> Seconds {
        self.start_fade_duration
    }

    pub fn end_fade_duration(&mut self) -> Seconds {
        self.end_fade_duration
    }

    fn to_proc_info(
        &self,
        sample_rate: SampleRate,
        timeline_start: SampleTime,
        timeline_end: SampleTime,
    ) -> AudioClipFadesProcInfo {
        let start_fade_duration = self
            .start_fade_duration
            .to_nearest_sample_round(sample_rate)
            .0 as usize;
        let end_fade_duration = self
            .end_fade_duration
            .to_nearest_sample_round(sample_rate)
            .0 as usize;

        let start_fade_delta = if start_fade_duration > 0 {
            1.0 / start_fade_duration as f32
        } else {
            0.0
        };

        let end_fade_delta = if end_fade_duration > 0 {
            1.0 / end_fade_duration as f32
        } else {
            0.0
        };

        AudioClipFadesProcInfo {
            start_fade_duration,
            end_fade_duration,

            start_fade_delta,
            end_fade_delta,

            start_fade_timeline_end: timeline_start + SampleTime::from_usize(start_fade_duration),
            end_fade_timeline_start: timeline_end - SampleTime::from_usize(end_fade_duration),
        }
    }
}

impl Default for AudioClipFades {
    fn default() -> Self {
        Self {
            start_fade_duration: Self::DEFAULT_FADE_DURATION,
            end_fade_duration: Self::DEFAULT_FADE_DURATION,
        }
    }
}

#[derive(Clone)]
struct AudioClipFadesProcInfo {
    start_fade_duration: usize,
    end_fade_duration: usize,

    start_fade_delta: f32,
    end_fade_delta: f32,

    start_fade_timeline_end: SampleTime,
    end_fade_timeline_start: SampleTime,
}

#[derive(Debug, Clone)]
pub struct AudioClipSaveState {
    name: String,
    pcm_path: PathBuf,
    timeline_start: MusicalTime,
    duration: Seconds,
    clip_start_offset: Seconds,
    clip_gain_db: f32,
    fades: AudioClipFades,
}

impl AudioClipSaveState {
    /// Create a new audio clip save state.
    ///
    /// * `name` - The name displayed on the audio clip.
    /// * `pcm_path` - The path to the audio file containing the PCM data.
    /// * `timeline_start` - Where the clip starts on the timeline.
    /// * `duration` - The duration of the clip on the timeline. If this is negative,
    /// then `0` seconds will be used instead.
    /// * `clip_start_offset` - The offset in the pcm resource where the "start" of the clip should start playing from.
    /// * `clip_gain_db` - The gain of the audio clip in decibels.
    pub fn new(
        name: String,
        pcm_path: PathBuf,
        timeline_start: MusicalTime,
        duration: Seconds,
        clip_start_offset: Seconds,
        clip_gain_db: f32,
        fades: AudioClipFades,
    ) -> Self {
        let duration = if duration.0 < 0.0 {
            Seconds(0.0)
        } else {
            duration
        };

        Self {
            name,
            pcm_path,
            timeline_start,
            duration,
            clip_start_offset,
            clip_gain_db,
            fades,
        }
    }

    /// The name displayed on the audio clip.
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// The path to the audio file containing the PCM data.
    #[inline]
    pub fn pcm_path(&self) -> &PathBuf {
        &self.pcm_path
    }

    /// Where the clip starts on the timeline.
    #[inline]
    pub fn timeline_start(&self) -> MusicalTime {
        self.timeline_start
    }

    /// The duration of the clip on the timeline.
    #[inline]
    pub fn duration(&self) -> Seconds {
        self.duration
    }

    /// The offset in the pcm resource where the "start" of the clip should start playing from.
    #[inline]
    pub fn clip_start_offset(&self) -> Seconds {
        self.clip_start_offset
    }

    /// The gain of the audio clip in decibels.
    #[inline]
    pub fn clip_gain_db(&self) -> f32 {
        self.clip_gain_db
    }

    /// The fades on this audio clip.
    #[inline]
    pub fn fades(&self) -> AudioClipFades {
        self.fades
    }
}

pub struct AudioClipHandle {
    clip_gain_db: ParamF32Handle,

    info: Shared<SharedCell<AudioClipProcInfo>>,
    coll_handle: Handle,
}

impl AudioClipHandle {
    /// Set the name displayed on this audio clip.
    pub fn set_name(&mut self, name: String, save_state: &mut AudioClipSaveState) {
        save_state.name = name;
    }

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
        new_info.fades = save_state.fades.to_proc_info(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
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
        new_info.fades = save_state.fades.to_proc_info(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
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

    pub fn set_fades(
        &mut self,
        fades: AudioClipFades,
        tempo_map: &TempoMap,
        save_state: &mut AudioClipSaveState,
    ) {
        save_state.fades = fades;

        let mut new_info = AudioClipProcInfo::clone(&self.info.get());
        new_info.fades = fades.to_proc_info(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
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
        new_info.fades = save_state.fades.to_proc_info(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }
}

struct AudioClipParams {
    pub clip_gain_amp: ParamF32,
}

#[derive(Clone)]
pub(super) struct AudioClipProcInfo {
    pub(super) timeline_start: SampleTime,
    pub(super) timeline_end: SampleTime,

    // Audio clip resources are always immutable. This reflects the non-destructive nature
    // of this sampler engine.
    resource: Shared<AudioClipResource>,

    clip_start_offset: SampleTime,

    fades: AudioClipFadesProcInfo,
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
                    fades: save_state.fades.to_proc_info(
                        tempo_map.sample_rate,
                        timeline_start,
                        timeline_end,
                    ),
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

        let amp = if amp.is_smoothing() {
            Some(amp)
        } else if amp[0] != 1.0 {
            Some(amp)
        } else {
            // Don't need to apply gain if amp is 1.0.
            None
        };

        // Apply gain to the samples and add them to the output.
        //
        // TODO: SIMD optimizations.
        simd::process_fallback(
            playhead,
            &*info,
            out,
            amp,
            copy_out_offset,
            pcm_start,
            skip,
            copy_frames,
        )
    }
}

mod simd {
    use super::{AnyPcm, AudioClipProcInfo, StereoAudioBlockBuffer};
    use crate::backend::{parameter::SmoothOutput, MAX_BLOCKSIZE};
    use rusty_daw_time::SampleTime;

    pub(super) fn process_fallback(
        playhead: SampleTime,
        info: &AudioClipProcInfo,
        out: &mut StereoAudioBlockBuffer,
        amp: Option<SmoothOutput<f32>>,
        copy_out_offset: usize,
        pcm_start: usize,
        skip: usize,
        frames: usize,
    ) {
        // Hint to compiler to optimize loops.
        let frames = frames.min(MAX_BLOCKSIZE);

        // Calculate fades.
        let mut do_fades = false;
        let (mut start_fade_amp, start_fade_delta) =
            if playhead >= info.timeline_start && playhead < info.fades.start_fade_timeline_end {
                do_fades = true;

                (
                    (playhead - info.timeline_start).0 as f32 * info.fades.start_fade_delta,
                    info.fades.start_fade_delta,
                )
            } else {
                (1.0, 0.0)
            };
        let (mut end_fade_amp, end_fade_delta) =
            if playhead >= info.fades.end_fade_timeline_start && playhead < info.timeline_end {
                do_fades = true;

                (
                    1.0 - ((playhead - info.fades.end_fade_timeline_start).0 as f32
                        * info.fades.end_fade_delta),
                    info.fades.end_fade_delta,
                )
            } else {
                (1.0, 0.0)
            };

        let out_left = &mut out.left[copy_out_offset..copy_out_offset + frames];
        let out_right = &mut out.right[copy_out_offset..copy_out_offset + frames];
        match &*info.resource.pcm {
            AnyPcm::Mono(pcm) => {
                let src = &pcm.data()[pcm_start..pcm_start + frames];

                if let Some(amp) = amp {
                    // Hint to compiler to optimize loops.
                    let skip = skip.min(MAX_BLOCKSIZE - frames);

                    if do_fades {
                        for i in 0..frames {
                            let amp = &amp.values[skip..skip + frames];

                            let total_amp = amp[i] * start_fade_amp * end_fade_amp;

                            out_left[i] += src[i] * total_amp;
                            out_right[i] += src[i] * total_amp;

                            start_fade_amp = (start_fade_amp + start_fade_delta).min(1.0);
                            end_fade_amp = (end_fade_amp - end_fade_delta).max(0.0);
                        }
                    } else {
                        for i in 0..frames {
                            let amp = &amp.values[skip..skip + frames];

                            out_left[i] += src[i] * amp[i];
                            out_right[i] += src[i] * amp[i];
                        }
                    }
                } else {
                    if do_fades {
                        for i in 0..frames {
                            let total_amp = start_fade_amp * end_fade_amp;

                            out_left[i] += src[i] * total_amp;
                            out_right[i] += src[i] * total_amp;

                            start_fade_amp = (start_fade_amp + start_fade_delta).min(1.0);
                            end_fade_amp = (end_fade_amp - end_fade_delta).max(0.0);
                        }
                    } else {
                        for i in 0..frames {
                            out_left[i] += src[i];
                            out_right[i] += src[i];
                        }
                    }
                }
            }
            AnyPcm::Stereo(pcm) => {
                let src_left = &pcm.left()[pcm_start..pcm_start + frames];
                let src_right = &pcm.right()[pcm_start..pcm_start + frames];

                if let Some(amp) = amp {
                    // Hint to compiler to optimize loops.
                    let skip = skip.min(MAX_BLOCKSIZE - frames);

                    if do_fades {
                        for i in 0..frames {
                            let amp = &amp.values[skip..skip + frames];

                            let total_amp = amp[i] * start_fade_amp * end_fade_amp;

                            out_left[i] += src_left[i] * total_amp;
                            out_right[i] += src_right[i] * total_amp;

                            start_fade_amp = (start_fade_amp + start_fade_delta).min(1.0);
                            end_fade_amp = (end_fade_amp - end_fade_delta).max(0.0);
                        }
                    } else {
                        for i in 0..frames {
                            let amp = &amp.values[skip..skip + frames];

                            out_left[i] += src_left[i] * amp[i];
                            out_right[i] += src_right[i] * amp[i];
                        }
                    }
                } else {
                    if do_fades {
                        for i in 0..frames {
                            let total_amp = start_fade_amp * end_fade_amp;

                            out_left[i] += src_left[i] * total_amp;
                            out_right[i] += src_right[i] * total_amp;

                            start_fade_amp = (start_fade_amp + start_fade_delta).min(1.0);
                            end_fade_amp = (end_fade_amp - end_fade_delta).max(0.0);
                        }
                    } else {
                        for i in 0..frames {
                            out_left[i] += src_left[i];
                            out_right[i] += src_right[i];
                        }
                    }
                }
            }
        }
    }
}
