
use basedrop::Shared;
use dropseed::plugin::HostRequestChannelSender;
use dropseed::plugin::{
    buffer::EventBuffer, ext, HostInfo, PluginActivatedInfo, PluginAudioThread, PluginDescriptor,
    PluginFactory, PluginInstanceID, PluginMainThread, ProcBuffers, ProcInfo, ProcessStatus,
};
use meadowlark_core_types::time::SampleRate;

pub static TIMELINE_TRACK_PLUG_RDN: &str = "app.meadowlark.timeline-track";

pub struct TimelineTrackPlugFactory;

impl PluginFactory for TimelineTrackPlugFactory {
    fn description(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: TIMELINE_TRACK_PLUG_RDN.into(),
            version: "0.1".into(),
            name: "Sample Browser".into(),
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

pub struct TimelineTrackPlugHandle {}

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
        _min_samples: u32,
        max_samples: u32,
        coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String> {
        Ok(PluginActivatedInfo {
            audio_thread: Box::new(TimelineTrackPlugAudioThread {}),
            internal_handle: Some(Box::new(TimelineTrackPlugHandle {})),
        })
    }

    fn audio_ports_ext(&mut self) -> Result<ext::audio_ports::PluginAudioPortsExt, String> {
        Ok(ext::audio_ports::PluginAudioPortsExt::stereo_out())
    }
}

pub struct TimelineTrackPlugAudioThread {}

impl PluginAudioThread for TimelineTrackPlugAudioThread {
    fn start_processing(&mut self) -> Result<(), ()> {
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
        ProcessStatus::Continue
    }

    fn param_flush(&mut self, in_events: &EventBuffer, _out_events: &mut EventBuffer) {}
}