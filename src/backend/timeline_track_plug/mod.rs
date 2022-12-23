use std::error::Error;

use basedrop::{Shared, SharedCell};
use dropseed::plugin_api::{
    buffer::EventBuffer, ext, HostInfo, HostRequestChannelSender, PluginActivatedInfo,
    PluginDescriptor, PluginFactory, PluginInstanceID, PluginMainThread, PluginProcessor,
    ProcBuffers, ProcInfo, ProcessStatus,
};
use meadowlark_core_types::time::SampleRate;

use super::audio_clip_renderer::AudioClipRenderer;

// TODO: Have tracks support a variable number of channels.

pub static TIMELINE_TRACK_PLUG_RDN: &str = "app.meadowlark.timeline-track";

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
    pub audio_clips_shared: Shared<SharedCell<Vec<AudioClipRenderer>>>,
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
        sample_rate: SampleRate,
        _min_frames: u32,
        max_frames: u32,
        coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String> {
        let audio_clips_shared =
            Shared::new(coll_handle, SharedCell::new(Shared::new(coll_handle, Vec::new())));

        Ok(PluginActivatedInfo {
            processor: Box::new(TimelineTrackPlugProcessor {
                audio_clips_shared: Shared::clone(&audio_clips_shared),
                temp_audio_clip_buffer: vec![
                    Vec::with_capacity(max_frames as usize),
                    Vec::with_capacity(max_frames as usize),
                ],
            }),
            internal_handle: Some(Box::new(TimelineTrackPlugHandle { audio_clips_shared })),
        })
    }

    fn audio_ports_ext(&mut self) -> Result<ext::audio_ports::PluginAudioPortsExt, String> {
        Ok(ext::audio_ports::PluginAudioPortsExt::stereo_out())
    }

    fn update_tempo_map(
        &mut self,
        new_tempo_map: &Shared<dropseed::plugin_api::transport::TempoMap>,
    ) {
    }
}

pub struct TimelineTrackPlugProcessor {
    audio_clips_shared: Shared<SharedCell<Vec<AudioClipRenderer>>>,
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

        let audio_clips = SharedCell::get(&*self.audio_clips_shared);

        for audio_clip_renderer in audio_clips.iter() {
            if proc_info.transport.is_range_active(
                audio_clip_renderer.timeline_start(),
                audio_clip_renderer.timeline_end(),
            ) {
                let frame_in_clip = proc_info.transport.playhead_frame().0 as i64
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
