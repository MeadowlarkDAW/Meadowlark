use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};
use fnv::FnvHashMap;
use rusty_daw_time::{SampleRate, SampleTime, Seconds, TempoMap};
use std::sync::{Arc, Mutex};

use crate::backend::audio_graph::{
    AudioGraphNode, MonoAudioBlockBuffer, ProcInfo, StereoAudioBlockBuffer,
};
use crate::backend::parameter::Smooth;
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

#[derive(Debug)]
pub struct TimelineTrackSaveState {
    /// The ID (name) of the timeline track. This must be unique for
    /// each timeline track.
    pub id: String,

    /// The audio clips in this track.
    pub audio_clips: Vec<AudioClipSaveState>,
}

pub struct TimelineTrackHandle {
    audio_clip_indexes: FnvHashMap<String, usize>,
    audio_clip_handles: Vec<AudioClipHandle>,

    process: Shared<SharedCell<TimelineTrackProcess>>,

    sample_rate: SampleRate,
    coll_handle: Handle,
}

impl TimelineTrackHandle {
    /// Return an immutable handle to the audio clip with given ID.
    pub fn audio_clip<'a>(
        &'a self,
        id: &String,
        save_state: &'a TimelineTrackSaveState,
    ) -> Option<(&'a AudioClipHandle, &'a AudioClipSaveState)> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            Some((
                &self.audio_clip_handles[*audio_clip_index],
                &save_state.audio_clips[*audio_clip_index],
            ))
        } else {
            None
        }
    }

    /// Return a mutable handle to the audio clip with given ID.
    pub fn audio_clip_mut<'a>(
        &'a mut self,
        id: &String,
        save_state: &'a mut TimelineTrackSaveState,
    ) -> Option<(&'a mut AudioClipHandle, &'a mut AudioClipSaveState)> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.get(id) {
            Some((
                &mut self.audio_clip_handles[*audio_clip_index],
                &mut save_state.audio_clips[*audio_clip_index],
            ))
        } else {
            None
        }
    }

    /// Set the ID of the audio clip. The audio clip's ID is used as the name. It must be unique for this track.
    ///
    /// TODO: Return custom error.
    pub fn set_audio_clip_id(
        &mut self,
        old_id: &String,
        new_id: String,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<(), ()> {
        if self.audio_clip_indexes.contains_key(&new_id) {
            return Err(());
        }

        if let Some(audio_clip_index) = self.audio_clip_indexes.remove(old_id) {
            self.audio_clip_indexes
                .insert(new_id.clone(), audio_clip_index);

            // Update the values in the save state.
            save_state.audio_clips[audio_clip_index].id = new_id;

            Ok(())
        } else {
            Err(())
        }
    }

    /// Add a new audio clip to this track. The ID must be unique for this track.
    ///
    /// TODO: Return custom error.
    pub fn add_audio_clip(
        &mut self,
        clip: AudioClipSaveState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
        cache: &Arc<Mutex<AudioClipResourceCache>>,
        tempo_map: &TempoMap,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<Result<(), PcmLoadError>, ()> {
        if self.audio_clip_indexes.contains_key(&clip.id) {
            return Err(());
        }

        let audio_clip_index = save_state.audio_clips.len();
        self.audio_clip_indexes
            .insert(clip.id.clone(), audio_clip_index);

        let (audio_clip_proc, params_handle, res) = AudioClipProcess::new(
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
        self.process
            .set(Shared::new(&self.coll_handle, new_process));

        self.audio_clip_handles.push(params_handle);
        save_state.audio_clips.push(clip);

        Ok(res)
    }

    /// Remove an audio clip from this track.
    pub fn remove_audio_clip(
        &mut self,
        id: &String,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<(), ()> {
        if let Some(audio_clip_index) = self.audio_clip_indexes.remove(id) {
            save_state.audio_clips.remove(audio_clip_index);
            self.audio_clip_handles.remove(audio_clip_index);

            // Shift every clip's index that appears after this one.
            for (_, clip_index) in self.audio_clip_indexes.iter_mut() {
                if *clip_index > audio_clip_index {
                    *clip_index -= 1;
                }
            }

            // Compile the new process.

            let mut new_process = TimelineTrackProcess::clone(&self.process.get());

            // Clone the old processes.
            let mut new_audio_clip_procs = Vec::clone(&new_process.audio_clips);

            // Remove the old clip.
            new_audio_clip_procs.remove(audio_clip_index);

            // Use the new processes.
            new_process.audio_clips = Shared::new(&self.coll_handle, new_audio_clip_procs);
            self.process
                .set(Shared::new(&self.coll_handle, new_process));

            Ok(())
        } else {
            Err(())
        }
    }

    pub(super) fn update_tempo_map(
        &mut self,
        tempo_map: &TempoMap,
        save_state: &TimelineTrackSaveState,
    ) {
        for (clip, save) in self
            .audio_clip_handles
            .iter_mut()
            .zip(save_state.audio_clips.iter())
        {
            clip.update_tempo_map(tempo_map, save);
        }
    }
}

pub struct TimelineTrackNode {
    process: Shared<SharedCell<TimelineTrackProcess>>,
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
        let mut audio_clip_indexes = FnvHashMap::<String, usize>::default();
        let mut audio_clip_handles = Vec::<AudioClipHandle>::new();

        for (audio_clip_index, audio_clip_save) in save_state.audio_clips.iter().enumerate() {
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
            audio_clip_indexes.insert(audio_clip_save.id.clone(), audio_clip_index);
            audio_clip_handles.push(handle);
        }

        let process = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                TimelineTrackProcess {
                    audio_clips: Shared::new(&coll_handle, audio_clip_procs),
                },
            )),
        );

        (
            Self {
                process: Shared::clone(&process),
            },
            TimelineTrackHandle {
                audio_clip_indexes,
                audio_clip_handles,
                process,
                sample_rate,
                coll_handle,
            },
            audio_clip_errors,
        )
    }

    fn audio_clips_loop_crossfade_out(
        frames: usize,
        loop_crossfade_out: &SmoothOutput<f32>,
        loop_out_playhead: SampleTime,
        process: &Shared<TimelineTrackProcess>,
        out: &mut AtomicRefMut<StereoAudioBlockBuffer>,
        crossfade_offset: usize,
    ) {
        // Tell compiler we want to optimize loops. (The min() condition should never actually happen.)
        let frames = frames.min(MAX_BLOCKSIZE);
        let crossfade_offset = crossfade_offset.min(frames);

        // Use a temporary buffer.
        //
        // This is safe because both this method and the audio_clip's `process()` method only reads the given
        // range of frames from [0..frames) (which is initialized to 0.0).
        let mut temp_out = unsafe { StereoAudioBlockBuffer::new_partially_uninit(0..frames) };

        let end_frame = loop_out_playhead + SampleTime::from_usize(frames);

        for audio_clip in process.audio_clips.iter() {
            let info = audio_clip.info.get();
            // Only use audio clips that lie within range of the current process cycle.
            if loop_out_playhead < info.timeline_end && info.timeline_start < end_frame {
                // Fill samples from the audio clip into the output buffer.
                audio_clip.process(loop_out_playhead, frames, &mut temp_out, 0);
            }
        }

        // Add all frames up to `crossfade_offset` to the output buffer.
        for i in 0..crossfade_offset {
            out.left[i] += temp_out.left[i];
            out.right[i] += temp_out.right[i];
        }

        // Add and declick (fade out) all frames after the `crossfade_offset`.
        for i in 0..(frames - crossfade_offset) {
            out.left[crossfade_offset + i] +=
                temp_out.left[crossfade_offset + i] * loop_crossfade_out[i];
            out.right[crossfade_offset + i] +=
                temp_out.right[crossfade_offset + i] * loop_crossfade_out[i];
        }
    }

    fn audio_clips_seek_crossfade_out(
        frames: usize,
        seek_crossfade_out: &SmoothOutput<f32>,
        seek_out_playhead: SampleTime,
        process: &Shared<TimelineTrackProcess>,
        out: &mut AtomicRefMut<StereoAudioBlockBuffer>,
    ) {
        // Tell compiler we want to optimize loops. (The min() condition should never actually happen.)
        let frames = frames.min(MAX_BLOCKSIZE);

        // Use a temporary buffer.
        //
        // This is safe because both this method and the audio_clip's `process()` method only reads the given
        // range of frames from [0..frames) (which is initialized to 0.0).
        //let mut temp_out = unsafe { StereoAudioBlockBuffer::new_partially_uninit(0..frames) };
        let mut temp_out = unsafe { StereoAudioBlockBuffer::new_partially_uninit(0..frames) };

        let end_frame = seek_out_playhead + SampleTime::from_usize(frames);

        for audio_clip in process.audio_clips.iter() {
            let info = audio_clip.info.get();
            // Only use audio clips that lie within range.
            if seek_out_playhead < info.timeline_end && info.timeline_start < end_frame {
                // Fill samples from the audio clip into the output buffer.
                audio_clip.process(seek_out_playhead, frames, &mut temp_out, 0);
            }
        }

        // Add and declick (fade out) all newly filled samples into the output buffer.
        for i in 0..frames {
            out.left[i] += temp_out.left[i] * seek_crossfade_out[i];
            out.right[i] += temp_out.right[i] * seek_crossfade_out[i];
        }
    }
}

impl AudioGraphNode for TimelineTrackNode {
    // TODO: Switch between mono and stereo based on the type of audio
    // clips on the track.
    fn stereo_audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        transport: &TimelineTransport,
        _mono_audio_in: &[AtomicRef<MonoAudioBlockBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioBlockBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioBlockBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioBlockBuffer>],
    ) {
        if !transport.audio_clip_declick().is_active() {
            // Nothing to do.
            return;
        }

        // Keep playing if there is an active pause/stop fade out.
        let playhead = transport
            .audio_clip_declick()
            .stop_fade_playhead()
            .unwrap_or(transport.playhead());

        let process = self.process.get();
        let stereo_out = &mut stereo_audio_out[0];

        // ----------------------------------------------------------------------------------
        // First, we fill the output buffer with samples from the audio clips.

        let loop_crossfade_in = transport.audio_clip_declick().loop_crossfade_in();
        let (loop_crossfade_out, loop_out_playhead) =
            transport.audio_clip_declick().loop_crossfade_out();

        if let Some(loop_back) = transport.do_loop_back() {
            // Transport is currently looping in this process cycle. We will need to process
            // loop crossfades individually.

            let first_frames = (loop_back.loop_end - playhead).0 as usize;
            let second_frames = proc_info.frames() - first_frames;

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

            // Declick (fade in) the newly filled samples.
            for i in 0..second_frames {
                stereo_out.left[first_frames + i] *= loop_crossfade_in[i];
                stereo_out.right[first_frames + i] *= loop_crossfade_in[i];
            }

            // Next, add in samples from the loop crossfade out.
            //
            // This is separated out because this method allocates a whole new audio
            // buffer on the stack.
            Self::audio_clips_loop_crossfade_out(
                proc_info.frames(),
                &loop_crossfade_out,
                loop_out_playhead,
                &process,
                stereo_out,
                // Tells this method to only start fading samples after this offset.
                first_frames,
            );
        } else {
            // Transport is not looping in this process cycle. Process in one chunk.

            let end_frame = playhead + SampleTime::from_usize(proc_info.frames());

            for audio_clip in process.audio_clips.iter() {
                let info = audio_clip.info.get();
                // Only use audio clips that lie within range of the current process cycle.
                if playhead < info.timeline_end && info.timeline_start < end_frame {
                    // Fill samples from the audio clip into the output buffer.
                    audio_clip.process(playhead, proc_info.frames(), stereo_out, 0);
                }
            }

            if loop_crossfade_in.is_smoothing() {
                // Declick (fade in) the newly filled samples.
                for i in 0..proc_info.frames() {
                    stereo_out.left[i] *= loop_crossfade_in[i];
                    stereo_out.right[i] *= loop_crossfade_in[i];
                }
            }

            if loop_crossfade_out.is_smoothing() {
                // Add in samples from any remaining loop crossfade outs.
                //
                // This is separated out because this method allocates a whole new audio
                // buffer on the stack.
                Self::audio_clips_loop_crossfade_out(
                    proc_info.frames(),
                    &loop_crossfade_out,
                    // Tells this method to start copying samples from where the previous
                    // loop out crossfade ended.
                    loop_out_playhead,
                    &process,
                    stereo_out,
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
            for i in 0..proc_info.frames() {
                stereo_out.left[i] *= seek_crossfade_in[i];
                stereo_out.right[i] *= seek_crossfade_in[i];
            }
        }

        if seek_crossfade_out.is_smoothing() {
            // Next, add in samples for the crossfade out.
            //
            // This is separated out because this method allocates a whole new audio
            // buffer on the stack.
            Self::audio_clips_seek_crossfade_out(
                proc_info.frames(),
                &seek_crossfade_out,
                seek_out_playhead,
                &process,
                stereo_out,
            );
        }

        // ----------------------------------------------------------------------------------
        // Finally, we apply start/stop declicking to the entire output buffer.

        let start_stop_fade = transport.audio_clip_declick().start_stop_fade();

        if start_stop_fade.is_smoothing() {
            // Declick (fade in/out) the filled samples.
            for i in 0..proc_info.frames() {
                stereo_out.left[i] *= start_stop_fade[i];
                stereo_out.right[i] *= start_stop_fade[i];
            }
        }
    }
}

#[derive(Clone)]
pub struct TimelineTrackProcess {
    audio_clips: Shared<Vec<AudioClipProcess>>,
}
