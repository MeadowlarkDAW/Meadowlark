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
        let (mut out_l, mut out_r) = buffers.audio_out[0].stereo_f32_mut().unwrap();
        let out_l = &mut out_l[0..proc_info.frames];
        let out_r = &mut out_r[0..proc_info.frames];

        // Clear the output buffers.
        out_l.fill(0.0);
        out_r.fill(0.0);

        if !proc_info.transport.is_playing() {
            return ProcessStatus::Continue;
        }

        let (temp_buf_l, temp_buf_r) = self.temp_audio_clip_buffer.split_first_mut().unwrap();
        let temp_buf_r = &mut temp_buf_r[0];
        let temp_buf_l = &mut temp_buf_l[0..proc_info.frames];
        let temp_buf_r = &mut temp_buf_r[0..proc_info.frames];

        let state = SharedCell::get(&*self.shared_state);

        for audio_clip_renderer in state.audio_clip_renderers.iter() {
            if proc_info.transport.is_range_active(
                audio_clip_renderer.timeline_start().0,
                audio_clip_renderer.timeline_end().0,
            ) {
                let frame_in_clip = proc_info.transport.playhead_frame() as i64
                    - audio_clip_renderer.timeline_start().0 as i64;

                if audio_clip_renderer.render_stereo(frame_in_clip, temp_buf_l, temp_buf_r) {
                    // Add the rendered samples to the final output.
                    for i in 0..proc_info.frames {
                        out_l[i] += temp_buf_l[i];
                        out_r[i] += temp_buf_r[i];
                    }
                }
            }
        }

        ProcessStatus::Continue
    }

    fn param_flush(&mut self, in_events: &EventBuffer, _out_events: &mut EventBuffer) {}
}
