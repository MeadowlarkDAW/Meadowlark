use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_audio_graph::node::{DB_GRADIENT, SMOOTH_SECS};
use rusty_daw_core::block_buffer::StereoBlockBuffer;
use rusty_daw_core::{
    Frames, MusicalTime, ParamF32, ParamF32UiHandle, ProcFrames, SampleRate, SuperFrames, Unit,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::backend::resource_loader::{AnyPcm, PcmLoadError, ResourceLoader};
use crate::backend::ResourceCache;

use super::{AudioClipState, TempoMap};

mod declick;
mod resource;

pub use declick::{AudioClipDeclick, DEFAULT_AUDIO_CLIP_DECLICK_TIME};
pub use resource::{AudioClipResource, AudioClipResourceCache};

pub static AUDIO_CLIP_GAIN_MIN_DB: f32 = -40.0;
pub static AUDIO_CLIP_GAIN_MAX_DB: f32 = 40.0;

#[derive(Debug, Clone, Copy)]
pub struct AudioClipFades {
    pub start_fade_duration: SuperFrames,
    pub end_fade_duration: SuperFrames,
}

impl AudioClipFades {
    pub const DEFAULT_FADE_DURATION: SuperFrames =
        SuperFrames(((10.0 / 1_000.0) * 508_032_000.0) as u64);

    pub fn no_fade() -> Self {
        Self { start_fade_duration: SuperFrames(0), end_fade_duration: SuperFrames(0) }
    }

    pub fn set_start_fade_duration(&mut self, duration: SuperFrames) {
        self.start_fade_duration = duration;
    }

    pub fn set_end_fade_duration(&mut self, duration: SuperFrames) {
        self.end_fade_duration = duration;
    }

    pub fn set_default_start_fade(&mut self) {
        self.start_fade_duration = Self::DEFAULT_FADE_DURATION;
    }

    pub fn set_default_end_fade(&mut self) {
        self.end_fade_duration = Self::DEFAULT_FADE_DURATION;
    }

    fn to_proc_state(
        &self,
        sample_rate: SampleRate,
        timeline_start: Frames,
        timeline_end: Frames,
    ) -> AudioClipFadesProcState {
        let start_fade_duration = self.start_fade_duration.to_nearest_frame_round(sample_rate);
        let mut end_fade_duration = self.end_fade_duration.to_nearest_frame_round(sample_rate);

        let end_fade_timeline_start = if timeline_end >= end_fade_duration {
            timeline_end - end_fade_duration
        } else {
            // Unlikely situation to happen, but still possible so avoid wrap-around bugs.
            end_fade_duration = timeline_end;
            Frames(0)
        };

        let start_fade_delta =
            if start_fade_duration.0 > 0 { 1.0 / start_fade_duration.0 as f32 } else { 0.0 };

        let end_fade_delta =
            if end_fade_duration.0 > 0 { 1.0 / end_fade_duration.0 as f32 } else { 0.0 };

        AudioClipFadesProcState {
            start_fade_duration,
            end_fade_duration,

            start_fade_delta,
            end_fade_delta,

            start_fade_timeline_end: timeline_start + start_fade_duration,
            end_fade_timeline_start,
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
struct AudioClipFadesProcState {
    start_fade_duration: Frames,
    end_fade_duration: Frames,

    start_fade_delta: f32,
    end_fade_delta: f32,

    start_fade_timeline_end: Frames,
    end_fade_timeline_start: Frames,
}

pub struct AudioClipHandle {
    clip_gain_db: ParamF32UiHandle,

    info: Shared<SharedCell<AudioClipProcState>>,
    coll_handle: Handle,
}

impl AudioClipHandle {
    /// Set the name displayed on this audio clip.
    pub fn set_name(&mut self, name: String, state: &mut AudioClipState) {
        state.name = name;
    }

    /// Set the gain of this audio clip.
    ///
    /// Returns the gain (this may be clamped to fit within range of the gain parameter).
    pub fn set_clip_gain_db(&mut self, gain_db: f32, state: &mut AudioClipState) -> f32 {
        self.clip_gain_db.set_value(gain_db);

        // Make sure value is clamped within range.
        let gain_db = self.clip_gain_db.value();
        state.clip_gain_db = gain_db;

        gain_db
    }

    /// Set where the clip starts on the timeline.
    pub fn set_timeline_start(
        &mut self,
        timeline_start: MusicalTime,
        tempo_map: &TempoMap,
        state: &mut AudioClipState,
    ) {
        state.timeline_start = timeline_start;

        let mut new_info = AudioClipProcState::clone(&self.info.get());
        new_info.timeline_start = tempo_map.musical_to_nearest_frame_round(state.timeline_start);
        new_info.timeline_end =
            (tempo_map.musical_to_nearest_super_frame_round(state.timeline_start) + state.duration)
                .to_nearest_frame_round(tempo_map.sample_rate);
        new_info.fades = state.fades.to_proc_state(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    /// Set the duration of the clip on the timeline.
    pub fn set_duration(
        &mut self,
        duration: SuperFrames,
        tempo_map: &TempoMap,
        state: &mut AudioClipState,
    ) {
        state.duration = duration;

        let mut new_info = AudioClipProcState::clone(&self.info.get());
        new_info.timeline_end =
            (tempo_map.musical_to_nearest_super_frame_round(state.timeline_start) + state.duration)
                .to_nearest_frame_round(tempo_map.sample_rate);
        new_info.fades = state.fades.to_proc_state(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    /// Set the offset where the clip should start playing from.
    pub fn set_clip_start_offset(
        &mut self,
        clip_start_offset: SuperFrames,
        clip_start_offset_is_negative: bool,
        tempo_map: &TempoMap,
        state: &mut AudioClipState,
    ) {
        state.clip_start_offset = clip_start_offset;
        state.clip_start_offset_is_negative = clip_start_offset_is_negative;

        let mut new_info = AudioClipProcState::clone(&self.info.get());
        new_info.clip_start_offset =
            clip_start_offset.to_nearest_frame_round(tempo_map.sample_rate).0 as i64;
        if clip_start_offset_is_negative {
            new_info.clip_start_offset = -new_info.clip_start_offset;
        }

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    /// Set the PCM resource to use from the given path to an audio file.
    pub fn set_pcm(
        &mut self,
        pcm_path: PathBuf,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        cache: &Arc<Mutex<AudioClipResourceCache>>,
        state: &mut AudioClipState,
    ) -> Result<(), PcmLoadError> {
        let (resource, pcm_load_res) = { cache.lock().unwrap().cache(state, resource_loader) };

        state.pcm_path = pcm_path;

        let mut new_info = AudioClipProcState::clone(&self.info.get());
        new_info.resource = resource;

        self.info.set(Shared::new(&self.coll_handle, new_info));

        pcm_load_res
    }

    pub fn set_fades(
        &mut self,
        fades: AudioClipFades,
        tempo_map: &TempoMap,
        state: &mut AudioClipState,
    ) {
        state.fades = fades;

        let mut new_info = AudioClipProcState::clone(&self.info.get());
        new_info.fades = fades.to_proc_state(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }

    pub fn resource(&self) -> Shared<AudioClipResource> {
        Shared::clone(&self.info.get().resource)
    }

    pub(super) fn update_tempo_map(&mut self, tempo_map: &TempoMap, state: &AudioClipState) {
        let mut new_info = AudioClipProcState::clone(&self.info.get());
        new_info.timeline_start = tempo_map.musical_to_nearest_frame_round(state.timeline_start);
        new_info.timeline_end =
            (tempo_map.musical_to_nearest_super_frame_round(state.timeline_start) + state.duration)
                .to_nearest_frame_round(tempo_map.sample_rate);
        new_info.fades = state.fades.to_proc_state(
            tempo_map.sample_rate,
            new_info.timeline_start,
            new_info.timeline_end,
        );

        self.info.set(Shared::new(&self.coll_handle, new_info));
    }
}

struct AudioClipParams<const MAX_BLOCKSIZE: usize> {
    pub clip_gain_amp: ParamF32<MAX_BLOCKSIZE>,
}

#[derive(Clone)]
pub(super) struct AudioClipProcState {
    pub(super) timeline_start: Frames,
    pub(super) timeline_end: Frames,

    // Audio clip resources are always immutable. This reflects the non-destructive nature
    // of this sampler engine.
    resource: Shared<AudioClipResource>,

    clip_start_offset: i64, // i64 instead of frame because this can be negative.

    fades: AudioClipFadesProcState,
}

#[derive(Clone)]
pub struct AudioClipProcess<const MAX_BLOCKSIZE: usize> {
    // Wrapping params in a shared pointer so we can clone this struct when compiling
    // a new list of processes. This should never cause a panic because this struct is the
    // only place this is ever borrowed.
    params: Shared<AtomicRefCell<AudioClipParams<MAX_BLOCKSIZE>>>,

    pub(super) info: Shared<SharedCell<AudioClipProcState>>,
}

impl<const MAX_BLOCKSIZE: usize> AudioClipProcess<MAX_BLOCKSIZE> {
    pub fn new(
        state: &AudioClipState,
        resource_cache: &ResourceCache,
        tempo_map: &TempoMap,
        coll_handle: &Handle,
    ) -> (Self, AudioClipHandle, Result<(), PcmLoadError>) {
        let clip_gain_db = state.clip_gain_db.clamp(AUDIO_CLIP_GAIN_MIN_DB, AUDIO_CLIP_GAIN_MAX_DB);

        let (gain_amp, gain_handle) = ParamF32::from_value(
            clip_gain_db,
            AUDIO_CLIP_GAIN_MIN_DB,
            AUDIO_CLIP_GAIN_MAX_DB,
            DB_GRADIENT,
            Unit::Decibels,
            SMOOTH_SECS,
            tempo_map.sample_rate,
        );

        let (resource, pcm_load_res) = {
            resource_cache
                .audio_clip_resource_cache
                .lock()
                .unwrap()
                .cache(state, &resource_cache.resource_loader)
        };

        let timeline_start = tempo_map.musical_to_nearest_frame_round(state.timeline_start);
        let timeline_end = (tempo_map.musical_to_nearest_super_frame_round(state.timeline_start)
            + state.duration)
            .to_nearest_frame_round(tempo_map.sample_rate);

        let mut clip_start_offset =
            state.clip_start_offset.to_nearest_frame_round(tempo_map.sample_rate).0 as i64;
        if state.clip_start_offset_is_negative {
            clip_start_offset = -clip_start_offset;
        };

        let info = Shared::new(
            coll_handle,
            SharedCell::new(Shared::new(
                coll_handle,
                AudioClipProcState {
                    resource,
                    timeline_start,
                    timeline_end,
                    clip_start_offset,
                    fades: state.fades.to_proc_state(
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
                    AtomicRefCell::new(AudioClipParams { clip_gain_amp: gain_amp }),
                ),
                info: Shared::clone(&info),
            },
            AudioClipHandle { clip_gain_db: gain_handle, info, coll_handle: coll_handle.clone() },
            pcm_load_res,
        )
    }

    pub fn process(
        &self,
        playhead: Frames,
        proc_frames: ProcFrames<MAX_BLOCKSIZE>,
        out: &mut StereoBlockBuffer<f32, MAX_BLOCKSIZE>,
        out_offset: usize,
    ) {
        let info = self.info.get();

        if info.timeline_start > playhead {
            return;
        }

        let mut params = self.params.borrow_mut();
        let amp = params.clip_gain_amp.smoothed(proc_frames);

        let mut copy_frames = proc_frames.compiler_hint_frames();
        let mut copy_out_offset = out_offset;
        let mut skip = 0;

        // Find the sample to start reading from in the PCM resource.
        let pcm_start = playhead.0 as i64 - info.timeline_start.0 as i64 + info.clip_start_offset
            - info.resource.original_offset.0 as i64;

        if pcm_start >= info.resource.pcm.frames().0 as i64 {
            // Out of range. Do nothing (add silence).
            return;
        }

        let pcm_start = if pcm_start < 0 {
            if pcm_start + copy_frames as i64 <= 0 {
                // Out of range. Do nothing (add silence).
                return;
            }

            // Skip frames (insert silence) until there is data.
            skip = (0 - pcm_start) as usize;
            copy_frames -= skip;
            copy_out_offset += skip;

            0
        } else {
            pcm_start as usize
        };

        if pcm_start + copy_frames > info.resource.pcm.frames().0 as usize {
            // Skip frames (add silence) after the end of the resource.
            copy_frames = info.resource.pcm.frames().0 as usize - pcm_start;
        }

        let amp = if amp.is_smoothing() {
            Some(amp)
        } else if (amp[0] - 1.0).abs() < 0.00001 {
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
            copy_frames.into(),
        )
    }
}

mod simd {
    use super::{AnyPcm, AudioClipProcState, StereoBlockBuffer};
    use rusty_daw_core::{Frames, ProcFrames, SmoothOutputF32};

    pub(super) fn process_fallback<const MAX_BLOCKSIZE: usize>(
        playhead: Frames,
        info: &AudioClipProcState,
        out: &mut StereoBlockBuffer<f32, MAX_BLOCKSIZE>,
        amp: Option<SmoothOutputF32<MAX_BLOCKSIZE>>,
        copy_out_offset: usize,
        pcm_start: usize,
        skip: usize,
        proc_frames: ProcFrames<MAX_BLOCKSIZE>,
    ) {
        // Hint to compiler to optimize loops.
        let frames = proc_frames.compiler_hint_frames();

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
