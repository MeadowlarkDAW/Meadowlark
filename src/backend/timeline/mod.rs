use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_time::{SampleRate, SampleTime, TempoMap};
use std::sync::{Arc, Mutex};

use crate::backend::graph::{AudioGraphNode, ProcBuffers, ProcInfo, StereoBlockBuffer};
use crate::backend::resource_loader::{PcmLoadError, ResourceLoadError, ResourceLoader};
use crate::backend::MAX_BLOCKSIZE;

pub mod audio_clip;
pub use audio_clip::{AudioClipResource, AudioClipResourceCache, AudioClipSaveState};

pub mod transport;
pub use transport::{
    LoopState, TimelineTransport, TimelineTransportHandle, TimelineTransportSaveState,
};

use audio_clip::{AudioClipHandle, AudioClipProcess};

use super::parameter::SmoothOutput;

#[derive(Debug, Clone)]
pub struct TimelineTrackSaveState {
    name: String,
    audio_clips: Vec<AudioClipSaveState>,
}

impl TimelineTrackSaveState {
    /// Create a new timeline track save state.
    ///
    /// * `name` - The name displayed on this timeline track.
    /// * `audio_clips` - The audio clips on this track. These
    /// do not need to be in any particular order.
    pub fn new(name: String, audio_clips: Vec<AudioClipSaveState>) -> Self {
        Self { name, audio_clips }
    }

    /// The name displayed on this timeline track.
    #[inline]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// The audio clips on this timeline track. These may not be
    /// in any particular order.
    #[inline]
    pub fn audio_clips(&self) -> &[AudioClipSaveState] {
        self.audio_clips.as_slice()
    }
}

pub struct TimelineTrackHandle {
    audio_clip_handles: Vec<AudioClipHandle>,

    process: Shared<SharedCell<TimelineTrackProcess>>,

    sample_rate: SampleRate,
    coll_handle: Handle,
}

impl TimelineTrackHandle {
    /// Set the name displayed on this timeline track.
    pub fn set_name(&mut self, name: String, save_state: &mut TimelineTrackSaveState) {
        save_state.name = name;
    }

    /// Return an immutable handle to the audio clip with the given index.
    pub fn audio_clip<'a>(
        &'a self,
        index: usize,
        save_state: &'a TimelineTrackSaveState,
    ) -> Option<(&'a AudioClipHandle, &'a AudioClipSaveState)> {
        if let Some(audio_clip) = self.audio_clip_handles.get(index) {
            Some((audio_clip, &save_state.audio_clips[index]))
        } else {
            None
        }
    }

    /// Return a mutable handle to the audio clip with the given index.
    pub fn audio_clip_mut<'a>(
        &'a mut self,
        index: usize,
        save_state: &'a mut TimelineTrackSaveState,
    ) -> Option<(&'a mut AudioClipHandle, &'a mut AudioClipSaveState)> {
        if let Some(audio_clip) = self.audio_clip_handles.get_mut(index) {
            Some((audio_clip, &mut save_state.audio_clips[index]))
        } else {
            None
        }
    }

    /// Add a new audio clip to this track.
    pub fn add_audio_clip(
        &mut self,
        clip: AudioClipSaveState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        cache: &Arc<Mutex<AudioClipResourceCache>>,
        tempo_map: &TempoMap,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<(), PcmLoadError> {
        let (audio_clip_proc, params_handle, pcm_load_res) = AudioClipProcess::new(
            &clip,
            resource_loader,
            cache,
            tempo_map,
            self.coll_handle.clone(),
        );

        // Compile the new process.

        let mut new_process = TimelineTrackProcess::clone(&self.process.get());

        // Clone the old processes.
        let mut new_audio_clip_procs = Vec::clone(&new_process.audio_clips);

        // Add the new clip.
        new_audio_clip_procs.push(audio_clip_proc);

        // Use the new process info.
        new_process.audio_clips = Shared::new(&self.coll_handle, new_audio_clip_procs);
        self.process.set(Shared::new(&self.coll_handle, new_process));

        self.audio_clip_handles.push(params_handle);
        save_state.audio_clips.push(clip);

        pcm_load_res
    }

    /// Remove an audio clip from this track.
    pub fn remove_audio_clip(
        &mut self,
        index: usize,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<(), ()> {
        if index >= self.audio_clip_handles.len() {
            return Err(());
        }

        self.audio_clip_handles.remove(index);
        save_state.audio_clips.remove(index);

        // Compile the new process.

        let mut new_process = TimelineTrackProcess::clone(&self.process.get());

        // Clone the old processes.
        let mut new_audio_clip_procs = Vec::clone(&new_process.audio_clips);

        // Remove the old clip.
        new_audio_clip_procs.remove(index);

        // Use the new processes.
        new_process.audio_clips = Shared::new(&self.coll_handle, new_audio_clip_procs);
        self.process.set(Shared::new(&self.coll_handle, new_process));

        Ok(())
    }

    pub(super) fn update_tempo_map(
        &mut self,
        tempo_map: &TempoMap,
        save_state: &TimelineTrackSaveState,
    ) {
        for (clip, save) in self.audio_clip_handles.iter_mut().zip(save_state.audio_clips.iter()) {
            clip.update_tempo_map(tempo_map, save);
        }
    }
}

pub struct TimelineTrackNode {
    process: Shared<SharedCell<TimelineTrackProcess>>,
    temp_buffer: Shared<AtomicRefCell<StereoBlockBuffer<f32>>>,
}

impl TimelineTrackNode {
    pub fn new(
        save_state: &TimelineTrackSaveState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        cache: &Arc<Mutex<AudioClipResourceCache>>,
        tempo_map: &TempoMap,
        sample_rate: SampleRate,
        coll_handle: Handle,
    ) -> (Self, TimelineTrackHandle, Vec<ResourceLoadError>) {
        let mut audio_clip_procs = Vec::<AudioClipProcess>::new();
        let mut audio_clip_errors = Vec::<ResourceLoadError>::new();
        let mut audio_clip_handles = Vec::<AudioClipHandle>::new();

        for audio_clip_save in save_state.audio_clips.iter() {
            let (process, handle, res) = AudioClipProcess::new(
                audio_clip_save,
                resource_loader,
                cache,
                tempo_map,
                coll_handle.clone(),
            );

            if let Err(e) = res {
                audio_clip_errors.push(ResourceLoadError::PCM(e));
            }

            audio_clip_procs.push(process);
            audio_clip_handles.push(handle);
        }

        let process = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                TimelineTrackProcess { audio_clips: Shared::new(&coll_handle, audio_clip_procs) },
            )),
        );

        (
            Self {
                process: Shared::clone(&process),
                temp_buffer: Shared::new(
                    &coll_handle,
                    AtomicRefCell::new(StereoBlockBuffer::new()),
                ),
            },
            TimelineTrackHandle { audio_clip_handles, process, sample_rate, coll_handle },
            audio_clip_errors,
        )
    }

    fn audio_clips_loop_crossfade_out(
        frames: usize,
        loop_crossfade_out: &SmoothOutput<f32>,
        loop_out_playhead: SampleTime,
        process: &Shared<TimelineTrackProcess>,
        out: &mut StereoBlockBuffer<f32>,
        temp_out: &mut StereoBlockBuffer<f32>,
        crossfade_offset: usize,
    ) {
        // Tell compiler we want to optimize loops. (The min() condition should never actually happen.)
        let frames = frames.min(MAX_BLOCKSIZE);
        let crossfade_offset = crossfade_offset.min(frames);

        // Clear output buffers to 0.0 because audio clips will add their samples instead
        // of overwriting them.
        temp_out.clear_frames(frames);

        let end_frame = loop_out_playhead + SampleTime::from_usize(frames);

        for audio_clip in process.audio_clips.iter() {
            let info = audio_clip.info.get();
            // Only use audio clips that lie within range of the current process cycle.
            if loop_out_playhead < info.timeline_end && info.timeline_start < end_frame {
                // Fill samples from the audio clip into the output buffer.
                audio_clip.process(loop_out_playhead, frames, temp_out, 0);
            }
        }

        // Add all frames up to `crossfade_offset` to the output buffer.
        for i in 0..crossfade_offset {
            out.left[i] += temp_out.left[i];
            out.right[i] += temp_out.right[i];
        }

        // Add and declick (fade out) all frames after the `crossfade_offset`.
        loop_crossfade_out.optimized_multiply_then_add_offset_stereo(
            &temp_out,
            out,
            frames - crossfade_offset,
            crossfade_offset,
        );
    }

    fn audio_clips_seek_crossfade_out(
        frames: usize,
        seek_crossfade_out: &SmoothOutput<f32>,
        seek_out_playhead: SampleTime,
        process: &Shared<TimelineTrackProcess>,
        out: &mut StereoBlockBuffer<f32>,
        temp_out: &mut StereoBlockBuffer<f32>,
    ) {
        // Tell compiler we want to optimize loops. (The min() condition should never actually happen.)
        let frames = frames.min(MAX_BLOCKSIZE);

        // Clear output buffers to 0.0 because audio clips will add their samples instead
        // of overwriting them.
        temp_out.clear_frames(frames);

        let end_frame = seek_out_playhead + SampleTime::from_usize(frames);

        for audio_clip in process.audio_clips.iter() {
            let info = audio_clip.info.get();
            // Only use audio clips that lie within range.
            if seek_out_playhead < info.timeline_end && info.timeline_start < end_frame {
                // Fill samples from the audio clip into the output buffer.
                audio_clip.process(seek_out_playhead, frames, temp_out, 0);
            }
        }

        // Add and declick (fade out) all newly filled samples into the output buffer.
        seek_crossfade_out.optimized_multiply_then_add_stereo(&temp_out, out, frames);
    }
}

impl AudioGraphNode for TimelineTrackNode {
    // TODO: Switch between mono and stereo based on the type of audio
    // clips on the track.
    fn stereo_audio_out_ports(&self) -> u32 {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f32>,
    ) {
        if buffers.stereo_audio_out.is_empty() {
            // Nothing to do.
            return;
        }

        let frames = proc_info.frames();

        // Won't panic because we checked this was not empty earlier.
        let stereo_out = &mut *buffers.stereo_audio_out.buffer_mut(0).unwrap();

        // Clear output buffer to 0.0 because audio clips will add their samples instead
        // of overwriting them.
        stereo_out.clear_frames(frames);

        if !transport.audio_clip_declick().is_active() {
            // Nothing to do.
            return;
        }

        // Keep playing if there is an active pause/stop fade out.
        let playhead =
            transport.audio_clip_declick().stop_fade_playhead().unwrap_or(transport.playhead());

        let process = self.process.get();

        // ----------------------------------------------------------------------------------
        // First, we fill the output buffer with samples from the audio clips.

        let loop_crossfade_in = transport.audio_clip_declick().loop_crossfade_in();
        let (loop_crossfade_out, loop_out_playhead) =
            transport.audio_clip_declick().loop_crossfade_out();

        if let Some(loop_back) = transport.do_loop_back() {
            // Transport is currently looping in this process cycle. We will need to process
            // loop crossfades individually.

            let first_frames = (loop_back.loop_end - playhead).0 as usize;
            let second_frames = frames - first_frames;

            // First, process the crossfade in.
            for audio_clip in process.audio_clips.iter() {
                let info = audio_clip.info.get();
                // Only use audio clips that lie within range of the current process
                // cycle after the point where the loop jumps back.
                if loop_back.loop_start < info.timeline_end
                    && info.timeline_start < loop_back.playhead_end
                {
                    // Fill samples from the audio clip into the output buffer.
                    //
                    // Here we only want to start filling in the samples after the
                    // point where the loop jumps back.
                    // (hence `out_offset` is`first_frames`)
                    audio_clip.process(
                        loop_back.loop_start,
                        second_frames,
                        stereo_out,
                        first_frames,
                    );
                }
            }

            // Declick (fade in) the newly filled samples starting from `first_frames`.
            loop_crossfade_in.optimized_multiply_offset_stereo(
                stereo_out,
                second_frames,
                first_frames,
            );

            // This will not panic because this is the only method where this is borrowed.
            let temp_out = &mut *self.temp_buffer.borrow_mut();

            // Next, add in samples from the loop crossfade out.
            Self::audio_clips_loop_crossfade_out(
                frames,
                &loop_crossfade_out,
                loop_out_playhead,
                &process,
                stereo_out,
                temp_out,
                // Tells this method to only start fading samples after this offset.
                first_frames,
            );
        } else {
            // Transport is not looping in this process cycle. Process in one chunk.

            let end_frame = playhead + SampleTime::from_usize(frames);

            for audio_clip in process.audio_clips.iter() {
                let info = audio_clip.info.get();
                // Only use audio clips that lie within range of the current process cycle.
                if playhead < info.timeline_end && info.timeline_start < end_frame {
                    // Fill samples from the audio clip into the output buffer.
                    audio_clip.process(playhead, frames, stereo_out, 0);
                }
            }

            if loop_crossfade_in.is_smoothing() {
                // Declick (fade in) the newly filled samples.
                loop_crossfade_in.optimized_multiply_stereo(stereo_out, frames);
            }

            // This will not panic because this is the only method where this is borrowed.
            let temp_out = &mut *self.temp_buffer.borrow_mut();

            if loop_crossfade_out.is_smoothing() {
                // Add in samples from any remaining loop crossfade outs.
                Self::audio_clips_loop_crossfade_out(
                    frames,
                    &loop_crossfade_out,
                    // Tells this method to start copying samples from where the previous
                    // loop out crossfade ended.
                    loop_out_playhead,
                    &process,
                    stereo_out,
                    temp_out,
                    0,
                );
            }
        }

        // ----------------------------------------------------------------------------------
        // Now that we filled the output buffer with samples from the audio clips, we apply
        // seek declicking.

        let seek_crossfade_in = transport.audio_clip_declick().seek_crossfade_in();
        let (seek_crossfade_out, seek_out_playhead) =
            transport.audio_clip_declick().seek_crossfade_out();

        if seek_crossfade_in.is_smoothing() {
            // Declick (fade in) the filled samples.
            seek_crossfade_in.optimized_multiply_stereo(stereo_out, frames);
        }

        if seek_crossfade_out.is_smoothing() {
            // This will not panic because this is the only method where this is borrowed.
            let temp_out = &mut *self.temp_buffer.borrow_mut();

            // Next, add in samples for the crossfade out.
            Self::audio_clips_seek_crossfade_out(
                frames,
                &seek_crossfade_out,
                seek_out_playhead,
                &process,
                stereo_out,
                temp_out,
            );
        }

        // ----------------------------------------------------------------------------------
        // Finally, we apply start/stop declicking to the entire output buffer.

        let start_stop_fade = transport.audio_clip_declick().start_stop_fade();

        if start_stop_fade.is_smoothing() {
            // Declick (fade in/out) the filled samples.
            start_stop_fade.optimized_multiply_stereo(stereo_out, frames);
        }
    }
}

#[derive(Clone)]
pub struct TimelineTrackProcess {
    audio_clips: Shared<Vec<AudioClipProcess>>,
}
