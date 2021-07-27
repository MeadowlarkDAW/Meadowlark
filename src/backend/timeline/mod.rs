use atomic_refcell::{AtomicRef, AtomicRefMut};
use basedrop::{Handle, Shared, SharedCell};
use fnv::FnvHashMap;
use rusty_daw_time::{SampleRate, SampleTime, Seconds, TempoMap};
use std::sync::{Arc, Mutex};

use crate::backend::graph_interface::{
    AudioGraphNode, MonoAudioPortBuffer, ProcInfo, StereoAudioPortBuffer,
};
use crate::backend::parameter::Smooth;
use crate::backend::resource_loader::{PcmLoadError, ResourceLoadError, ResourceLoader};
use crate::backend::MAX_BLOCKSIZE;

pub mod audio_clip;
pub use audio_clip::AudioClipSaveState;

pub mod transport;
pub use transport::{
    LoopState, TimelineTransport, TimelineTransportHandle, TimelineTransportSaveState,
};

mod sampler;

use audio_clip::{AudioClipHandle, AudioClipProcess};

use self::transport::LoopBackInfo;

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
        tempo_map: &TempoMap,
        save_state: &mut TimelineTrackSaveState,
    ) -> Result<Result<(), PcmLoadError>, ()> {
        if self.audio_clip_indexes.contains_key(&clip.id) {
            return Err(());
        }

        let audio_clip_index = save_state.audio_clips.len();
        self.audio_clip_indexes
            .insert(clip.id.clone(), audio_clip_index);

        let (audio_clip_proc, params_handle, res) =
            AudioClipProcess::new(&clip, resource_loader, tempo_map, self.coll_handle.clone());

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
    sample_rate: SampleRate,

    process: Shared<SharedCell<TimelineTrackProcess>>,
}

impl TimelineTrackNode {
    pub fn new(
        save_state: &TimelineTrackSaveState,
        resource_loader: &Arc<Mutex<ResourceLoader>>,
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
                sample_rate,
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
        _mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        _mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        _stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    ) {
        let process = self.process.get();
        let stereo_out = &mut stereo_audio_out[0];

        if let Some(loop_back) = transport.do_loop_back() {
        } else {
            // Finish any remaining crossfade outs.
            let crossfade_out = transport.audio_clip_declick().crossfade_out.output();
            if crossfade_out.is_smoothing() {
                let playhead = transport.audio_clip_declick().crossfade_out_playhead();
                let end_frame = playhead + SampleTime(proc_info.frames() as i64);

                for audio_clip in process.audio_clips.iter() {
                    let info = audio_clip.info.get();
                    if playhead < info.timeline_end && info.timeline_start < end_frame {
                        audio_clip.process(
                            playhead,
                            proc_info.frames(),
                            proc_info.sample_rate,
                            &stereo_out,
                            0,
                        );
                    }
                }

                // Declick
                for i in 0..proc_info.frames() {
                    stereo_out.left[i] *= crossfade_out[i];
                    stereo_out.right[i] *= crossfade_out[i];
                }
            }

            let crossfade_in = transport.audio_clip_declick().crossfade_in();
            let global_gain = transport.audio_clip_declick().global_gain();
            if transport.is_playing() || crossfade_in.is_smoothing() || global_gain.is_smoothing() {
                // End frame is known because we checked that we are not looping.
                let end_frame = transport.playhead() + SampleTime(proc_info.frames() as i64);

                for audio_clip in process.audio_clips.iter() {
                    let info = audio_clip.info.get();
                    if transport.playhead() < info.timeline_end && info.timeline_start < end_frame {
                        audio_clip.process(
                            transport.playhead(),
                            proc_info.frames(),
                            proc_info.sample_rate,
                            &stereo_out,
                            0,
                        );
                    }
                }

                if crossfade_in.is_smoothing() {
                    // Declick
                    for i in 0..proc_info.frames() {
                        stereo_out.left[i] *= crossfade_in[i];
                        stereo_out.right[i] *= crossfade_in[i];
                    }
                }
                if global_gain.is_smoothing() {
                    // Declick
                    for i in 0..proc_info.frames() {
                        stereo_out.left[i] *= global_gain[i];
                        stereo_out.right[i] *= global_gain[i];
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct TimelineTrackProcess {
    audio_clips: Shared<Vec<AudioClipProcess>>,
}

pub struct AudioClipDeclick {
    global_gain: Smooth<f32>,

    crossfade_out: Smooth<f32>,
    crossfade_in: Smooth<f32>,

    crossfade_out_playhead: SampleTime,
    crossfade_out_next_playhead: SampleTime,

    playing: bool,
    smoothing: bool,
}

impl AudioClipDeclick {
    pub fn new(fade_time: Seconds, sample_rate: SampleRate) -> Self {
        let mut global_gain = Smooth::new(0.0);
        global_gain.set_speed(sample_rate, fade_time);

        let mut crossfade_out = Smooth::new(1.0);
        crossfade_out.set_speed(sample_rate, fade_time);

        let mut crossfade_in = Smooth::new(0.0);
        crossfade_in.set_speed(sample_rate, fade_time);

        Self {
            global_gain,
            crossfade_out,
            crossfade_in,
            crossfade_out_playhead: SampleTime(0),
            crossfade_out_next_playhead: SampleTime(0),
            playing: false,
            smoothing: false,
        }
    }

    pub fn process(&mut self, proc_info: &ProcInfo, timeline: &TimelineTransport) {
        if self.playing != timeline.is_playing() {
            self.playing = timeline.is_playing();
            self.smoothing = true;

            if self.playing {
                // Fade in.
                self.global_gain.set(1.0);
            } else {
                // Fade out.
                self.global_gain.set(0.0);
            }
        }

        self.global_gain.process(proc_info.frames());
        self.global_gain.update_status();

        if self.crossfade_in.is_active() {
            self.crossfade_in.process(proc_info.frames());
            self.crossfade_in.update_status();

            if !self.crossfade_in.is_active() {
                // Reset the crossfade.
                self.crossfade_in.reset(0.0);
            }
        }

        if self.crossfade_out.is_active() {
            self.crossfade_out.process(proc_info.frames());
            self.crossfade_out.update_status();

            self.crossfade_out_playhead = self.crossfade_out_next_playhead;
            self.crossfade_out_next_playhead += SampleTime::from(proc_info.frames() as i64);

            if !self.crossfade_out.is_active() {
                // Reset the crossfade.
                self.crossfade_out.reset(1.0);
            }
        }

        if let Some(loop_back) = timeline.do_loop_back() {
            let second_frames = loop_back.playhead_end.0 as usize - timeline.playhead().0 as usize;

            // Start crossfade.
            self.crossfade_in.set(1.0);
            self.crossfade_out.set(0.0);

            // Only process the second chunk of frames.

            self.crossfade_in.process(second_frames);
            self.crossfade_in.update_status();

            if !self.crossfade_in.is_active() {
                // Reset the crossfade.
                self.crossfade_in.reset(0.0);
            }

            self.crossfade_out.process(second_frames);
            self.crossfade_out.update_status();

            self.crossfade_out_playhead = timeline.playhead();
            self.crossfade_out_next_playhead = loop_back.loop_end;

            if !self.crossfade_out.is_active() {
                // Reset the crossfade.
                self.crossfade_out.reset(1.0);
            }
        }
    }

    fn global_gain(&self) -> SmoothOutput<f32> {
        self.global_gain.output()
    }

    fn crossfade_in(&self) -> SmoothOutput<f32> {
        self.crossfade_in.output()
    }

    fn crossfade_out(&self) -> SmoothOutput<f32> {
        self.crossfade_out.output()
    }

    fn crossfade_out_playhead(&self) -> SampleTime {
        self.crossfade_out_playhead
    }

    fn is_active(&self) -> bool {
        self.global_gain.is_active()
            || self.crossfade_in.is_active()
            || self.crossfade_out.is_active()
    }
}
