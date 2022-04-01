use atomic_refcell::AtomicRefCell;
use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_audio_graph::{AudioGraphNode, ProcBuffers, ProcInfo};
use rusty_daw_core::block_buffer::StereoBlockBuffer;
use rusty_daw_core::{Frames, ProcFrames, SampleRate, SmoothOutputF32};

use crate::backend::resource_loader::{PcmLoadError, ResourceLoadError};
use crate::backend::ResourceCache;

use super::{
    AudioClipHandle, AudioClipProcess, AudioClipState, TempoMap, TimelineTrackState,
    TimelineTransport,
};

pub struct TimelineTrackHandle<const MAX_BLOCKSIZE: usize> {
    audio_clip_handles: Vec<AudioClipHandle>,

    process: Shared<SharedCell<TimelineTrackProcess<MAX_BLOCKSIZE>>>,

    sample_rate: SampleRate,
    coll_handle: Handle,
}

impl<const MAX_BLOCKSIZE: usize> TimelineTrackHandle<MAX_BLOCKSIZE> {
    /// Set the name displayed on this timeline track.
    pub fn set_name(&mut self, name: String, state: &mut TimelineTrackState) {
        state.name = name;
    }

    /// Return an immutable handle to the audio clip with the given index.
    pub fn audio_clip<'a>(
        &'a self,
        index: usize,
        state: &'a TimelineTrackState,
    ) -> Option<(&'a AudioClipHandle, &'a AudioClipState)> {
        if let Some(audio_clip) = self.audio_clip_handles.get(index) {
            Some((audio_clip, &state.audio_clips[index]))
        } else {
            None
        }
    }

    /// Return a mutable handle to the audio clip with the given index.
    pub fn audio_clip_mut<'a>(
        &'a mut self,
        index: usize,
        state: &'a mut TimelineTrackState,
    ) -> Option<(&'a mut AudioClipHandle, &'a mut AudioClipState)> {
        if let Some(audio_clip) = self.audio_clip_handles.get_mut(index) {
            Some((audio_clip, &mut state.audio_clips[index]))
        } else {
            None
        }
    }

    /// Add a new audio clip to this track.
    pub fn add_audio_clip(
        &mut self,
        clip: AudioClipState,
        resource_cache: &ResourceCache,
        tempo_map: &TempoMap,
        state: &mut TimelineTrackState,
    ) -> Result<(), PcmLoadError> {
        let (audio_clip_proc, params_handle, pcm_load_res) =
            AudioClipProcess::new(&clip, resource_cache, tempo_map, &self.coll_handle);

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
        state.audio_clips.push(clip);

        pcm_load_res
    }

    /// Remove an audio clip from this track.
    pub fn remove_audio_clip(
        &mut self,
        index: usize,
        state: &mut TimelineTrackState,
    ) -> Result<(), ()> {
        if index >= self.audio_clip_handles.len() {
            return Err(());
        }

        self.audio_clip_handles.remove(index);
        state.audio_clips.remove(index);

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

    pub(super) fn update_tempo_map(&mut self, tempo_map: &TempoMap, state: &TimelineTrackState) {
        for (clip, save) in self.audio_clip_handles.iter_mut().zip(state.audio_clips.iter()) {
            clip.update_tempo_map(tempo_map, save);
        }
    }
}

pub struct TimelineTrackNode<const MAX_BLOCKSIZE: usize> {
    process: Shared<SharedCell<TimelineTrackProcess<MAX_BLOCKSIZE>>>,
    temp_buffer: Shared<AtomicRefCell<StereoBlockBuffer<f32, MAX_BLOCKSIZE>>>,
}

impl<const MAX_BLOCKSIZE: usize> TimelineTrackNode<MAX_BLOCKSIZE> {
    pub fn new(
        state: &TimelineTrackState,
        resource_cache: &ResourceCache,
        tempo_map: &TempoMap,
        coll_handle: &Handle,
    ) -> (Self, TimelineTrackHandle<MAX_BLOCKSIZE>, Vec<ResourceLoadError>) {
        let mut audio_clip_procs = Vec::<AudioClipProcess<MAX_BLOCKSIZE>>::new();
        let mut audio_clip_errors = Vec::<ResourceLoadError>::new();
        let mut audio_clip_handles = Vec::<AudioClipHandle>::new();

        for audio_clip_save in state.audio_clips.iter() {
            let (process, handle, res) =
                AudioClipProcess::new(audio_clip_save, resource_cache, tempo_map, coll_handle);

            if let Err(e) = res {
                audio_clip_errors.push(ResourceLoadError::PCM(e));
            }

            audio_clip_procs.push(process);
            audio_clip_handles.push(handle);
        }

        let process = Shared::new(
            coll_handle,
            SharedCell::new(Shared::new(
                coll_handle,
                TimelineTrackProcess { audio_clips: Shared::new(coll_handle, audio_clip_procs) },
            )),
        );

        (
            Self {
                process: Shared::clone(&process),
                temp_buffer: Shared::new(coll_handle, AtomicRefCell::new(StereoBlockBuffer::new())),
            },
            TimelineTrackHandle {
                audio_clip_handles,
                process,
                sample_rate: tempo_map.sample_rate,
                coll_handle: coll_handle.clone(),
            },
            audio_clip_errors,
        )
    }

    fn audio_clips_loop_crossfade_out(
        proc_frames: ProcFrames<MAX_BLOCKSIZE>,
        loop_crossfade_out: &SmoothOutputF32<MAX_BLOCKSIZE>,
        loop_out_playhead: Frames,
        process: &Shared<TimelineTrackProcess<MAX_BLOCKSIZE>>,
        out: &mut StereoBlockBuffer<f32, MAX_BLOCKSIZE>,
        temp_out: &mut StereoBlockBuffer<f32, MAX_BLOCKSIZE>,
        crossfade_offset: usize,
    ) {
        // Tell compiler we want to optimize loops. (The min() condition should never actually happen.)
        let frames = proc_frames.compiler_hint_frames();
        let crossfade_offset = crossfade_offset.min(frames);

        // Clear output buffers to 0.0 because audio clips will add their samples instead
        // of overwriting them.
        temp_out.clear_frames(proc_frames);

        let end_frame = loop_out_playhead + Frames::from_proc_frames(proc_frames);

        for audio_clip in process.audio_clips.iter() {
            let info = audio_clip.info.get();
            // Only use audio clips that lie within range of the current process cycle.
            if loop_out_playhead < info.timeline_end && info.timeline_start < end_frame {
                // Fill samples from the audio clip into the output buffer.
                audio_clip.process(loop_out_playhead, proc_frames, temp_out, 0);
            }
        }

        // Add all frames up to `crossfade_offset` to the output buffer.
        for i in 0..crossfade_offset {
            out.left[i] += temp_out.left[i];
            out.right[i] += temp_out.right[i];
        }

        // Add and declick (fade out) all frames after the `crossfade_offset`.
        let proc_frames = frames - crossfade_offset;
        if loop_crossfade_out.is_smoothing() {
            for i in 0..proc_frames {
                out.left[crossfade_offset + i] +=
                    temp_out.left[crossfade_offset + i] * loop_crossfade_out[i];
                out.right[crossfade_offset + i] +=
                    temp_out.right[crossfade_offset + i] * loop_crossfade_out[i];
            }
        } else {
            // We can optimize by using a constant gain (better SIMD load efficiency).
            let gain_amp = loop_crossfade_out[0];

            for i in 0..proc_frames {
                out.left[crossfade_offset + i] += temp_out.left[crossfade_offset + i] * gain_amp;
                out.right[crossfade_offset + i] += temp_out.right[crossfade_offset + i] * gain_amp;
            }
        }
    }

    fn audio_clips_seek_crossfade_out(
        proc_frames: ProcFrames<MAX_BLOCKSIZE>,
        seek_crossfade_out: &SmoothOutputF32<MAX_BLOCKSIZE>,
        seek_out_playhead: Frames,
        process: &Shared<TimelineTrackProcess<MAX_BLOCKSIZE>>,
        out: &mut StereoBlockBuffer<f32, MAX_BLOCKSIZE>,
        temp_out: &mut StereoBlockBuffer<f32, MAX_BLOCKSIZE>,
    ) {
        // Tell compiler we want to optimize loops. (The min() condition should never actually happen.)
        let frames = proc_frames.compiler_hint_frames();

        // Clear output buffers to 0.0 because audio clips will add their samples instead
        // of overwriting them.
        temp_out.clear_frames(proc_frames);

        let end_frame = seek_out_playhead + Frames::from_proc_frames(proc_frames);

        for audio_clip in process.audio_clips.iter() {
            let info = audio_clip.info.get();
            // Only use audio clips that lie within range.
            if seek_out_playhead < info.timeline_end && info.timeline_start < end_frame {
                // Fill samples from the audio clip into the output buffer.
                audio_clip.process(seek_out_playhead, proc_frames, temp_out, 0);
            }
        }

        for i in 0..frames {
            out.left[i] += temp_out.left[i] * seek_crossfade_out[i];
            out.right[i] += temp_out.right[i] * seek_crossfade_out[i];
        }
    }
}

pub trait TimelineGlobalData<const MAX_BLOCKSIZE: usize>: Send + Sync + 'static {
    fn timeline_transport(&self) -> &TimelineTransport<MAX_BLOCKSIZE>;
}

impl<G: TimelineGlobalData<MAX_BLOCKSIZE>, const MAX_BLOCKSIZE: usize>
    AudioGraphNode<G, MAX_BLOCKSIZE> for TimelineTrackNode<MAX_BLOCKSIZE>
{
    fn debug_name(&self) -> &'static str {
        "TimelineTrackNode"
    }

    // TODO: Switch between mono and stereo based on the type of audio
    // clips on the track.
    fn indep_stereo_out_ports(&self) -> u32 {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo<MAX_BLOCKSIZE>,
        buffers: ProcBuffers<f32, MAX_BLOCKSIZE>,
        global_data: &G,
    ) {
        if buffers.indep_stereo_out.is_empty() {
            // Nothing to do.
            return;
        }

        let stereo_out = &mut *buffers.indep_stereo_out[0].atomic_borrow_mut();

        // Clear output buffer to 0.0 because audio clips will add their samples instead
        // of overwriting them.
        stereo_out.clear_frames(proc_info.frames);

        let transport = global_data.timeline_transport();

        // Only active when the transport is playing/fading out.
        if !transport.audio_clip_declick().is_active() {
            return;
        }

        let frames = proc_info.frames.compiler_hint_frames();

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
                        second_frames.into(),
                        stereo_out,
                        first_frames,
                    );
                }
            }

            // Declick (fade in) the newly filled samples starting after the `first_frames`.
            if loop_crossfade_in.is_smoothing() {
                for i in 0..second_frames {
                    stereo_out.left[first_frames + i] *= loop_crossfade_in[i];
                    stereo_out.right[first_frames + i] *= loop_crossfade_in[i];
                }
            } else {
                // We can optimize by using a constant gain (better SIMD load efficiency).
                let gain_amp = loop_crossfade_in[0];

                for i in 0..second_frames {
                    stereo_out.left[first_frames + i] *= gain_amp;
                    stereo_out.right[first_frames + i] *= gain_amp;
                }
            }

            // This will not panic because this is the only method where this is borrowed.
            let temp_out = &mut *self.temp_buffer.borrow_mut();

            // Next, add in samples from the loop crossfade out.
            Self::audio_clips_loop_crossfade_out(
                proc_info.frames,
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

            let end_frame = playhead + Frames::from_proc_frames(proc_info.frames);

            for audio_clip in process.audio_clips.iter() {
                let info = audio_clip.info.get();
                // Only use audio clips that lie within range of the current process cycle.
                if playhead < info.timeline_end && info.timeline_start < end_frame {
                    // Fill samples from the audio clip into the output buffer.
                    audio_clip.process(playhead, proc_info.frames, stereo_out, 0);
                }
            }

            if loop_crossfade_in.is_smoothing() {
                // Declick (fade in) the newly filled samples.
                for i in 0..frames {
                    stereo_out.left[i] *= loop_crossfade_in[i];
                    stereo_out.right[i] *= loop_crossfade_in[i];
                }
            }

            // This will not panic because this is the only method where this is borrowed.
            let temp_out = &mut *self.temp_buffer.borrow_mut();

            if loop_crossfade_out.is_smoothing() {
                // Add in samples from any remaining loop crossfade outs.
                Self::audio_clips_loop_crossfade_out(
                    proc_info.frames,
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
            for i in 0..frames {
                stereo_out.left[i] *= seek_crossfade_in[i];
                stereo_out.right[i] *= seek_crossfade_in[i];
            }
        }

        if seek_crossfade_out.is_smoothing() {
            // This will not panic because this is the only method where this is borrowed.
            let temp_out = &mut *self.temp_buffer.borrow_mut();

            // Next, add in samples for the crossfade out.
            Self::audio_clips_seek_crossfade_out(
                proc_info.frames,
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
            for i in 0..frames {
                stereo_out.left[i] *= start_stop_fade[i];
                stereo_out.right[i] *= start_stop_fade[i];
            }
        }
    }
}

#[derive(Clone)]
pub struct TimelineTrackProcess<const MAX_BLOCKSIZE: usize> {
    audio_clips: Shared<Vec<AudioClipProcess<MAX_BLOCKSIZE>>>,
}
