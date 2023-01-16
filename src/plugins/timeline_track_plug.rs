use basedrop::{Shared, SharedCell};
use meadowlark_plugin_api::{
    buffer::EventBuffer, ext, HostInfo, HostRequestChannelSender, PluginActivatedInfo,
    PluginDescriptor, PluginFactory, PluginInstanceID, PluginMainThread, PluginProcessor,
    ProcBuffers, ProcInfo, ProcessStatus,
};
use std::error::Error;

mod audio_clip_renderer;
pub use audio_clip_renderer::AudioClipRenderer;

use crate::resource::ResourceLoader;
use crate::state_system::source_state::{
    AudioClipCopyableState, AudioClipState, ProjectTrackState, TrackType,
};
use crate::state_system::time::TempoMap;

// TODO: Have tracks support a variable number of channels.

pub static TIMELINE_TRACK_PLUG_RDN: &str = "app.meadowlark.timeline-track";

pub struct TimelineTrackPlugState {
    pub shared_audio_clip_renderers: Shared<SharedCell<Vec<AudioClipRenderer>>>,
}

impl Clone for TimelineTrackPlugState {
    fn clone(&self) -> Self {
        Self { shared_audio_clip_renderers: Shared::clone(&self.shared_audio_clip_renderers) }
    }
}

pub struct TimelineTrackPlugFactory;

impl PluginFactory for TimelineTrackPlugFactory {
    fn description(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: TIMELINE_TRACK_PLUG_RDN.into(),
            version: "0.1".into(),
            name: "Timeline Track".into(),
            vendor: "Meadowlark".into(),
            description: String::new(),
            url: String::new(),
            manual_url: String::new(),
            support_url: String::new(),
            features: String::new(),
        }
    }

    fn instantiate(
        &mut self,
        host_request_channel: HostRequestChannelSender,
        _host_info: Shared<HostInfo>,
        _plugin_id: PluginInstanceID,
        _coll_handle: &basedrop::Handle,
    ) -> Result<Box<dyn PluginMainThread>, String> {
        Ok(Box::new(TimelineTrackPlugMainThread::new(host_request_channel)))
    }
}

pub struct TimelineTrackPlugHandle {
    shared_state: TimelineTrackPlugState,
    coll_handle: basedrop::Handle,
}

impl TimelineTrackPlugHandle {
    pub fn sync_from_track_state(
        &mut self,
        state: &ProjectTrackState,
        tempo_map: &TempoMap,
        resource_loader: &mut ResourceLoader,
    ) {
        self.sync_all_audio_clips(state, tempo_map, resource_loader)
    }

    pub fn sync_all_audio_clips(
        &mut self,
        state: &ProjectTrackState,
        tempo_map: &TempoMap,
        resource_loader: &mut ResourceLoader,
    ) {
        if let TrackType::Audio(audio_track_state) = &state.type_ {
            let mut audio_clip_renderers: Vec<AudioClipRenderer> =
                Vec::with_capacity(audio_track_state.clips.len());

            for audio_clip_state in audio_track_state.clips.iter() {
                audio_clip_renderers.push(AudioClipRenderer::new(
                    &audio_clip_state,
                    tempo_map,
                    resource_loader,
                ));
            }

            self.shared_state
                .shared_audio_clip_renderers
                .set(Shared::new(&self.coll_handle, audio_clip_renderers));
        }
    }

    pub fn sync_audio_clip_copyable_states(
        &mut self,
        clip_indexes_and_states: &[(usize, AudioClipCopyableState)],
        tempo_map: &TempoMap,
    ) {
        let mut audio_clip_renderers: Vec<AudioClipRenderer> =
            (*self.shared_state.shared_audio_clip_renderers.get()).clone();

        for (clip_index, new_state) in clip_indexes_and_states.iter() {
            if let Some(audio_clip_renderer) = audio_clip_renderers.get_mut(*clip_index) {
                audio_clip_renderer.sync_with_new_copyable_state(new_state, tempo_map);
            }
        }

        self.shared_state
            .shared_audio_clip_renderers
            .set(Shared::new(&self.coll_handle, audio_clip_renderers));
    }

    pub fn sync_audio_clip(
        &mut self,
        state: &ProjectTrackState,
        clip_index: usize,
        tempo_map: &TempoMap,
        resource_loader: &mut ResourceLoader,
    ) {
        if let TrackType::Audio(audio_track_state) = &state.type_ {
            if let Some(audio_clip_state) = audio_track_state.clips.get(clip_index) {
                let mut audio_clip_renderers: Vec<AudioClipRenderer> =
                    (*self.shared_state.shared_audio_clip_renderers.get()).clone();

                audio_clip_renderers[clip_index] =
                    AudioClipRenderer::new(&audio_clip_state, tempo_map, resource_loader);

                self.shared_state
                    .shared_audio_clip_renderers
                    .set(Shared::new(&self.coll_handle, audio_clip_renderers));
            }
        }
    }

    pub fn insert_audio_clip(
        &mut self,
        audio_clip_state: &AudioClipState,
        tempo_map: &TempoMap,
        resource_loader: &mut ResourceLoader,
    ) {
        let mut audio_clip_renderers: Vec<AudioClipRenderer> =
            (*self.shared_state.shared_audio_clip_renderers.get()).clone();

        audio_clip_renderers.push(AudioClipRenderer::new(
            audio_clip_state,
            tempo_map,
            resource_loader,
        ));

        self.shared_state
            .shared_audio_clip_renderers
            .set(Shared::new(&self.coll_handle, audio_clip_renderers));
    }

    pub fn remove_audio_clip(&mut self, index: usize) {
        let mut audio_clip_renderers: Vec<AudioClipRenderer> =
            (*self.shared_state.shared_audio_clip_renderers.get()).clone();

        if index < audio_clip_renderers.len() {
            audio_clip_renderers.remove(index);

            self.shared_state
                .shared_audio_clip_renderers
                .set(Shared::new(&self.coll_handle, audio_clip_renderers));
        }
    }
}

pub struct TimelineTrackPlugMainThread {
    host_request_channel: HostRequestChannelSender,
}

impl TimelineTrackPlugMainThread {
    pub fn new(host_request_channel: HostRequestChannelSender) -> Self {
        Self { host_request_channel }
    }
}

impl PluginMainThread for TimelineTrackPlugMainThread {
    fn activate(
        &mut self,
        sample_rate: u32,
        _min_frames: u32,
        max_frames: u32,
        coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String> {
        let shared_state = TimelineTrackPlugState {
            shared_audio_clip_renderers: Shared::new(
                coll_handle,
                SharedCell::new(Shared::new(coll_handle, Vec::new())),
            ),
        };

        Ok(PluginActivatedInfo {
            processor: Box::new(TimelineTrackPlugProcessor {
                shared_state: shared_state.clone(),
                temp_audio_clip_buffer: vec![
                    vec![0.0; max_frames as usize],
                    vec![0.0; max_frames as usize],
                ],
            }),
            internal_handle: Some(Box::new(TimelineTrackPlugHandle {
                shared_state,
                coll_handle: coll_handle.clone(),
            })),
        })
    }

    fn audio_ports_ext(&mut self) -> Result<ext::audio_ports::PluginAudioPortsExt, String> {
        Ok(ext::audio_ports::PluginAudioPortsExt::stereo_out())
    }
}

pub struct TimelineTrackPlugProcessor {
    shared_state: TimelineTrackPlugState,
    temp_audio_clip_buffer: Vec<Vec<f32>>,
}

impl TimelineTrackPlugProcessor {
    fn process_audio_clips(
        audio_clip_renderers: &[AudioClipRenderer],
        temp_audio_clip_buffer: &mut [Vec<f32>],
        proc_info: &ProcInfo,
        buffers: &mut ProcBuffers,
    ) -> bool {
        let (mut out_l_buf, mut out_r_buf) = buffers.audio_out[0].stereo_f32_mut().unwrap();
        let out_l = &mut out_l_buf.data[0..proc_info.frames];
        let out_r = &mut out_r_buf.data[0..proc_info.frames];

        // Clear the output buffers.
        out_l.fill(0.0);
        out_r.fill(0.0);

        let mut has_audio = false;

        let mut has_declick = false;
        let declick_info = proc_info.transport.declick_info();

        if declick_info.start_stop_active || declick_info.jump_active {
            has_declick = true;

            let (temp_buf_l, temp_buf_r) = temp_audio_clip_buffer.split_first_mut().unwrap();
            let temp_buf_r = &mut temp_buf_r[0];
            let temp_buf_l = &mut temp_buf_l[0..proc_info.frames];
            let temp_buf_r = &mut temp_buf_r[0..proc_info.frames];

            let declick_buffers = declick_info.buffers();

            let start_stop_declick_buf = if declick_info.start_stop_active {
                Some(&declick_buffers.start_stop_buf[0..proc_info.frames])
            } else {
                None
            };

            if declick_info.jump_active {
                // The playhead jumped to a new position (either from looping or from the user seeking
                // to a new position while the transport was playing), so we need to crossfade between
                // the old position and the new position.

                let declick_jump_out_buf = &declick_buffers.jump_out_buf[0..proc_info.frames];
                let declick_jump_in_buf = &declick_buffers.jump_in_buf[0..proc_info.frames];

                let jump_out_end_frame =
                    declick_info.jump_out_playhead_frame + proc_info.frames as u64;
                let jump_in_end_frame =
                    declick_info.jump_in_playhead_frame + proc_info.frames as i64;

                for audio_clip_renderer in audio_clip_renderers.iter() {
                    // Handle the audio clips which fall within the jump-out crossfade.
                    if declick_info.jump_out_playhead_frame < audio_clip_renderer.timeline_end().0
                        && audio_clip_renderer.timeline_start().0 < jump_out_end_frame
                    {
                        let frame_in_clip = declick_info.jump_out_playhead_frame as i64
                            - audio_clip_renderer.timeline_start().0 as i64;

                        let did_render_audio = audio_clip_renderer.render_stereo(
                            frame_in_clip,
                            temp_buf_l,
                            temp_buf_r,
                        );
                        if did_render_audio {
                            has_audio = true;

                            if let Some(start_stop_declick_buf) = start_stop_declick_buf {
                                // The transport just started/stopped, so declick by applying
                                // an ease in or ease out.

                                if proc_info.transport.is_playing()
                                    && declick_info.start_declick_start_frame
                                        <= audio_clip_renderer.timeline_start().0
                                {
                                    // If the audio clip happens to land on or after where the transport started, then
                                    // no transport-start declicking needs to occur. This is to preserve transients
                                    // when starting the transport at the beginning of an audio clip.

                                    for i in 0..proc_info.frames {
                                        out_l[i] += temp_buf_l[i] * declick_jump_out_buf[i];
                                        out_r[i] += temp_buf_r[i] * declick_jump_out_buf[i];
                                    }
                                } else {
                                    for i in 0..proc_info.frames {
                                        out_l[i] += temp_buf_l[i]
                                            * start_stop_declick_buf[i]
                                            * declick_jump_out_buf[i];
                                        out_r[i] += temp_buf_r[i]
                                            * start_stop_declick_buf[i]
                                            * declick_jump_out_buf[i];
                                    }
                                }
                            } else {
                                for i in 0..proc_info.frames {
                                    out_l[i] += temp_buf_l[i] * declick_jump_out_buf[i];
                                    out_r[i] += temp_buf_r[i] * declick_jump_out_buf[i];
                                }
                            }
                        }
                    }

                    // Handle the audio clips which fall within the jump-in crossfade.
                    if declick_info.jump_in_playhead_frame
                        < (audio_clip_renderer.timeline_end().0 as i64)
                        && (audio_clip_renderer.timeline_start().0 as i64) < jump_in_end_frame
                    {
                        let frame_in_clip = declick_info.jump_in_playhead_frame
                            - audio_clip_renderer.timeline_start().0 as i64;

                        let did_render_audio = audio_clip_renderer.render_stereo(
                            frame_in_clip,
                            temp_buf_l,
                            temp_buf_r,
                        );
                        if did_render_audio {
                            has_audio = true;

                            if declick_info.jump_in_declick_start_frame
                                <= audio_clip_renderer.timeline_start().0 as i64
                            {
                                // If the audio clip happens to land on or after where the transport looped back
                                // to, then no loop-in declicking needs to occur. This is to preserve transients
                                // when looping back to clips that are aligned to the start of the loop.

                                for i in 0..proc_info.frames {
                                    out_l[i] += temp_buf_l[i];
                                    out_r[i] += temp_buf_r[i];
                                }
                            } else {
                                for i in 0..proc_info.frames {
                                    out_l[i] += temp_buf_l[i] * declick_jump_in_buf[i];
                                    out_r[i] += temp_buf_r[i] * declick_jump_in_buf[i];
                                }
                            }
                        }
                    }
                }
            } else {
                // The transport just started/stopped, so declick by applying
                // an ease in or ease out.

                let start_stop_declick_buf = start_stop_declick_buf.unwrap();
                let playhead_end = proc_info.transport.playhead_frame() + proc_info.frames as u64;

                for audio_clip_renderer in audio_clip_renderers.iter() {
                    if proc_info.transport.playhead_frame() < audio_clip_renderer.timeline_end().0
                        && audio_clip_renderer.timeline_start().0 < playhead_end
                    {
                        let frame_in_clip = proc_info.transport.playhead_frame() as i64
                            - audio_clip_renderer.timeline_start().0 as i64;

                        let did_render_audio = audio_clip_renderer.render_stereo(
                            frame_in_clip,
                            temp_buf_l,
                            temp_buf_r,
                        );
                        if did_render_audio {
                            has_audio = true;

                            if proc_info.transport.is_playing()
                                && declick_info.start_declick_start_frame
                                    <= audio_clip_renderer.timeline_start().0
                            {
                                // If the audio clip happens to land on or after where the transport started, then
                                // no transport-start declicking needs to occur. This is to preserve transients
                                // when starting the transport at the beginning of an audio clip.

                                for i in 0..proc_info.frames {
                                    out_l[i] += temp_buf_l[i];
                                    out_r[i] += temp_buf_r[i];
                                }
                            } else {
                                for i in 0..proc_info.frames {
                                    out_l[i] += temp_buf_l[i] * start_stop_declick_buf[i];
                                    out_r[i] += temp_buf_r[i] * start_stop_declick_buf[i];
                                }
                            }
                        }
                    }
                }
            }
        }

        if !has_declick && proc_info.transport.is_playing() {
            let (temp_buf_l, temp_buf_r) = temp_audio_clip_buffer.split_first_mut().unwrap();
            let temp_buf_r = &mut temp_buf_r[0];
            let temp_buf_l = &mut temp_buf_l[0..proc_info.frames];
            let temp_buf_r = &mut temp_buf_r[0..proc_info.frames];

            for audio_clip_renderer in audio_clip_renderers.iter() {
                if proc_info.transport.is_range_active(
                    audio_clip_renderer.timeline_start().0,
                    audio_clip_renderer.timeline_end().0,
                ) {
                    let frame_in_clip = proc_info.transport.playhead_frame() as i64
                        - audio_clip_renderer.timeline_start().0 as i64;

                    let did_render_audio =
                        audio_clip_renderer.render_stereo(frame_in_clip, temp_buf_l, temp_buf_r);
                    if did_render_audio {
                        has_audio = true;

                        // Add the rendered samples to the final output.
                        for i in 0..proc_info.frames {
                            out_l[i] += temp_buf_l[i];
                            out_r[i] += temp_buf_r[i];
                        }
                    }
                }
            }
        }

        has_audio
    }
}

impl PluginProcessor for TimelineTrackPlugProcessor {
    fn start_processing(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn stop_processing(&mut self) {}

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        buffers: &mut ProcBuffers,
        in_events: &EventBuffer,
        _out_events: &mut EventBuffer,
    ) -> ProcessStatus {
        let audio_clip_renderers = self.shared_state.shared_audio_clip_renderers.get();
        if audio_clip_renderers.is_empty() {
            // This track has no audio clips, so fill the output with silence.
            buffers.clear_all_outputs_and_set_constant_hint(proc_info);
        } else {
            let has_audio = Self::process_audio_clips(
                &audio_clip_renderers,
                &mut self.temp_audio_clip_buffer,
                proc_info,
                buffers,
            );
            if !has_audio {
                buffers.set_constant_hint_on_all_outputs(true);
            }
        }

        ProcessStatus::Continue
    }

    fn param_flush(&mut self, in_events: &EventBuffer, _out_events: &mut EventBuffer) {}
}
