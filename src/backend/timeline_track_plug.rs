use std::error::Error;

use basedrop::{Shared, SharedCell};
use dropseed::plugin_api::{
    buffer::EventBuffer, ext, HostInfo, HostRequestChannelSender, PluginActivatedInfo,
    PluginDescriptor, PluginFactory, PluginInstanceID, PluginMainThread, PluginProcessor,
    ProcBuffers, ProcInfo, ProcessStatus,
};

use super::audio_clip_renderer::AudioClipRenderer;

// TODO: Have tracks support a variable number of channels.

pub static TIMELINE_TRACK_PLUG_RDN: &str = "app.meadowlark.timeline-track";

pub struct TimelineTrackPlugState {
    pub audio_clip_renderers: Vec<AudioClipRenderer>,
}

impl TimelineTrackPlugState {
    pub fn new() -> Self {
        Self { audio_clip_renderers: Vec::new() }
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
    shared_state: Shared<SharedCell<TimelineTrackPlugState>>,
    coll_handle: basedrop::Handle,
}

impl TimelineTrackPlugHandle {
    pub fn set_state(&mut self, state: TimelineTrackPlugState) {
        self.shared_state.set(Shared::new(&self.coll_handle, state));
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
        let shared_state = Shared::new(
            coll_handle,
            SharedCell::new(Shared::new(coll_handle, TimelineTrackPlugState::new())),
        );

        Ok(PluginActivatedInfo {
            processor: Box::new(TimelineTrackPlugProcessor {
                shared_state: Shared::clone(&shared_state),
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
    shared_state: Shared<SharedCell<TimelineTrackPlugState>>,
    temp_audio_clip_buffer: Vec<Vec<f32>>,
}

impl TimelineTrackPlugProcessor {
    fn process_audio_clips(
        &mut self,
        state: &Shared<TimelineTrackPlugState>,
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
        if let Some(declick_info) = proc_info.transport.declick_info() {
            if declick_info.start_stop_active || declick_info.jump_active {
                has_declick = true;

                let (temp_buf_l, temp_buf_r) =
                    self.temp_audio_clip_buffer.split_first_mut().unwrap();
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

                    for audio_clip_renderer in state.audio_clip_renderers.iter() {
                        // Handle the audio clips which fall within the jump-out crossfade.
                        if declick_info.jump_out_playhead_frame < audio_clip_renderer.timeline_end.0
                            && audio_clip_renderer.timeline_start.0 < jump_out_end_frame
                        {
                            let frame_in_clip = declick_info.jump_out_playhead_frame as i64
                                - audio_clip_renderer.timeline_start.0 as i64;

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
                                            <= audio_clip_renderer.timeline_start.0
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
                            < (audio_clip_renderer.timeline_end.0 as i64)
                            && (audio_clip_renderer.timeline_start.0 as i64) < jump_in_end_frame
                        {
                            let frame_in_clip = declick_info.jump_in_playhead_frame
                                - audio_clip_renderer.timeline_start.0 as i64;

                            let did_render_audio = audio_clip_renderer.render_stereo(
                                frame_in_clip,
                                temp_buf_l,
                                temp_buf_r,
                            );
                            if did_render_audio {
                                has_audio = true;

                                if declick_info.jump_in_declick_start_frame
                                    <= audio_clip_renderer.timeline_start.0 as i64
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
                    let playhead_end =
                        proc_info.transport.playhead_frame() + proc_info.frames as u64;

                    for audio_clip_renderer in state.audio_clip_renderers.iter() {
                        if proc_info.transport.playhead_frame() < audio_clip_renderer.timeline_end.0
                            && audio_clip_renderer.timeline_start.0 < playhead_end
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
                                        <= audio_clip_renderer.timeline_start.0
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
        }

        if !has_declick && proc_info.transport.is_playing() {
            let (temp_buf_l, temp_buf_r) = self.temp_audio_clip_buffer.split_first_mut().unwrap();
            let temp_buf_r = &mut temp_buf_r[0];
            let temp_buf_l = &mut temp_buf_l[0..proc_info.frames];
            let temp_buf_r = &mut temp_buf_r[0..proc_info.frames];

            for audio_clip_renderer in state.audio_clip_renderers.iter() {
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
        let state = SharedCell::get(&*self.shared_state);
        if state.audio_clip_renderers.is_empty() {
            // This track has no audio clips, so fill the output with silence.
            buffers.clear_all_outputs_and_set_constant_hint(proc_info);
        } else {
            let has_audio = self.process_audio_clips(&state, proc_info, buffers);
            if !has_audio {
                buffers.set_constant_hint_on_all_outputs(true);
            }
        }

        ProcessStatus::Continue
    }

    fn param_flush(&mut self, in_events: &EventBuffer, _out_events: &mut EventBuffer) {}
}
