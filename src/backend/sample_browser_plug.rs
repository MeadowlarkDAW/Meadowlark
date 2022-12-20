use basedrop::{Owned, Shared};
use dropseed::plugin_api::event::ParamValueEvent;
use dropseed::plugin_api::ext::params::{ParamID, ParamInfo, ParamInfoFlags};
use dropseed::plugin_api::{
    buffer::EventBuffer, ext, HostInfo, HostRequestChannelSender, HostRequestFlags,
    PluginActivatedInfo, PluginDescriptor, PluginFactory, PluginInstanceID, PluginMainThread,
    PluginProcessor, ProcBuffers, ProcInfo, ProcessStatus,
};
use meadowlark_core_types::parameter::{
    ParamF32, ParamF32Handle, Unit, DEFAULT_DB_GRADIENT, DEFAULT_SMOOTH_SECS,
};
use meadowlark_core_types::time::{SampleRate, SecondsF64};
use pcm_loader::PcmRAM;
use rtrb::{Consumer, Producer, RingBuffer};
use std::error::Error;
use std::fmt::Write;

pub static SAMPLE_BROWSER_PLUG_RDN: &str = "app.meadowlark.sample-browser";

static CROSSFADE_TIME: SecondsF64 = SecondsF64(3.0 / 1000.0);

const MSG_BUFFER_SIZE: usize = 64;

// TODO: Use disk streaming with `creek` for sample playback instead of loading
// the whole file upfront in the UI.

pub struct SampleBrowserPlugFactory;

impl PluginFactory for SampleBrowserPlugFactory {
    fn description(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: SAMPLE_BROWSER_PLUG_RDN.into(),
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
        Ok(Box::new(SampleBrowserPlugMainThread::new(host_request_channel)))
    }
}

pub struct SampleBrowserPlugHandle {
    to_processor_tx: Producer<ProcessMsg>,
    host_request: HostRequestChannelSender,
}

impl SampleBrowserPlugHandle {
    pub fn play_pcm(&mut self, pcm: Shared<PcmRAM>) {
        self.send(ProcessMsg::PlayPCM { pcm });
        self.host_request.request(HostRequestFlags::PROCESS);
    }

    /*
    pub fn replay_pcm(&mut self) {
        self.send(ProcessMsg::ReplayPCM);
        self.host_request.request(HostRequestFlags::PROCESS);
    }
    */

    pub fn stop(&mut self) {
        self.send(ProcessMsg::Stop);
    }

    fn send(&mut self, msg: ProcessMsg) {
        if let Err(e) = self.to_processor_tx.push(msg) {
            log::error!("Sample browser plugin failed to send message: {}", e);
        }
    }
}

enum ProcessMsg {
    PlayPCM { pcm: Shared<PcmRAM> },
    //ReplayPCM,
    Stop,
}

struct ParamsHandle {
    pub gain: ParamF32Handle,
}

struct Params {
    pub gain: ParamF32,
}

impl Params {
    fn new(sample_rate: SampleRate, max_frames: usize) -> (Self, ParamsHandle) {
        let (gain, gain_handle) = ParamF32::from_value(
            0.0,
            0.0,
            -90.0,
            0.0,
            DEFAULT_DB_GRADIENT,
            Unit::Decibels,
            DEFAULT_SMOOTH_SECS,
            sample_rate,
            max_frames,
        );

        (Params { gain }, ParamsHandle { gain: gain_handle })
    }
}

pub struct SampleBrowserPlugMainThread {
    params: ParamsHandle,
    host_request: HostRequestChannelSender,
}

impl SampleBrowserPlugMainThread {
    fn new(host_request: HostRequestChannelSender) -> Self {
        // These parameters will be re-initialized later with the correct sample_rate
        // and max_frames when the plugin is activated.
        let (_params, params_handle) = Params::new(Default::default(), 0);

        Self { params: params_handle, host_request }
    }
}

impl PluginMainThread for SampleBrowserPlugMainThread {
    fn activate(
        &mut self,
        sample_rate: SampleRate,
        _min_frames: u32,
        max_frames: u32,
        coll_handle: &basedrop::Handle,
    ) -> Result<PluginActivatedInfo, String> {
        let (params, params_handle) = Params::new(sample_rate, max_frames as usize);
        self.params = params_handle;

        let (to_processor_tx, from_handle_rx) = RingBuffer::<ProcessMsg>::new(MSG_BUFFER_SIZE);
        let from_handle_rx = Owned::new(coll_handle, from_handle_rx);

        let crossfade_frames = CROSSFADE_TIME.to_nearest_frame_round(sample_rate).0 as usize;
        let crossfade_inc = 1.0 / crossfade_frames as f32;

        let fade_out_buf_l = Owned::new(coll_handle, vec![0.0; max_frames as usize]);
        let fade_out_buf_r = Owned::new(coll_handle, vec![0.0; max_frames as usize]);

        Ok(PluginActivatedInfo {
            processor: Box::new(SampleBrowserPlugProcessor {
                params,
                from_handle_rx,
                play_state: PlayState::Stopped,
                fade_out_state: FadeOutState::Stopped,
                pcm: None,
                old_pcm: None,
                crossfade_inc,
                crossfade_frames,
                fade_out_buf_l,
                fade_out_buf_r,
                fade_in_frames_left: 0,
                fade_in_current_gain: 0.0,
            }),
            internal_handle: Some(Box::new(SampleBrowserPlugHandle {
                to_processor_tx,
                host_request: self.host_request.clone(),
            })),
        })
    }

    fn audio_ports_ext(&mut self) -> Result<ext::audio_ports::PluginAudioPortsExt, String> {
        Ok(ext::audio_ports::PluginAudioPortsExt::stereo_out())
    }

    // --- Parameters ---------------------------------------------------------------------------------

    fn num_params(&mut self) -> u32 {
        1
    }

    fn param_info(&mut self, param_index: usize) -> Result<ParamInfo, Box<dyn Error>> {
        match param_index {
            0 => Ok(ParamInfo::new(
                ParamID(0),
                ParamInfoFlags::default_float(),
                "gain".into(),
                String::new(),
                0.0,
                1.0,
                1.0,
            )),
            _ => Err(format!("Param at index {} does not exist", param_index).into()),
        }
    }

    fn param_value(&self, param_id: ParamID) -> Result<f64, Box<dyn Error>> {
        match param_id {
            ParamID(0) => Ok(f64::from(self.params.gain.normalized())),
            _ => Err(format!("Param with id {:?} does not exist", param_id).into()),
        }
    }

    fn param_value_to_text(
        &self,
        param_id: ParamID,
        value: f64,
        text_buffer: &mut String,
    ) -> Result<(), String> {
        match param_id {
            ParamID(0) => {
                let value = self.params.gain.normalized_to_value(value as f32);
                write!(text_buffer, "{:.2} dB", value).unwrap();
            }
            _ => return Err(String::new()),
        }
        Ok(())
    }

    fn param_text_to_value(&self, param_id: ParamID, text: &str) -> Option<f64> {
        match param_id {
            ParamID(0) => {
                if let Ok(value) = text.parse::<f32>() {
                    return Some(self.params.gain.value_to_normalized(value) as f64);
                }
            }
            _ => (),
        }
        None
    }
}

#[derive(Clone, Copy)]
enum PlayState {
    Stopped,
    Playing { playhead: usize },
}

#[derive(Clone, Copy)]
enum FadeOutState {
    Stopped,
    Running { old_playhead: usize, current_gain: f32, frames_left: usize },
}

pub struct SampleBrowserPlugProcessor {
    params: Params,

    from_handle_rx: Owned<Consumer<ProcessMsg>>,

    play_state: PlayState,
    fade_out_state: FadeOutState,

    pcm: Option<Shared<PcmRAM>>,
    old_pcm: Option<Shared<PcmRAM>>,

    crossfade_inc: f32,
    crossfade_frames: usize,

    fade_out_buf_l: Owned<Vec<f32>>,
    fade_out_buf_r: Owned<Vec<f32>>,

    fade_in_frames_left: usize,
    fade_in_current_gain: f32,
}

impl SampleBrowserPlugProcessor {
    fn poll(&mut self, in_events: &EventBuffer) {
        for e in in_events.iter() {
            if let Some(param_value) = e.as_event::<ParamValueEvent>() {
                if param_value.param_id() == 0 {
                    self.params.gain.set_normalized(param_value.value().clamp(0.0, 1.0) as f32);
                }
            }
        }

        while let Ok(msg) = self.from_handle_rx.pop() {
            match msg {
                ProcessMsg::PlayPCM { pcm } => {
                    if let PlayState::Playing { playhead: old_playhead } = self.play_state {
                        self.old_pcm = Some(self.pcm.take().unwrap());
                        self.pcm = Some(pcm);

                        self.fade_out_state = FadeOutState::Running {
                            old_playhead,
                            current_gain: 1.0,
                            frames_left: self.crossfade_frames,
                        };

                        self.fade_in_frames_left = self.crossfade_frames;
                        self.fade_in_current_gain = 0.0;

                        self.play_state = PlayState::Playing { playhead: 0 };
                    } else {
                        self.pcm = Some(pcm);

                        self.fade_in_frames_left = 0;

                        self.play_state = PlayState::Playing { playhead: 0 };
                    }
                }
                /*
                ProcessMsg::ReplayPCM => {
                    if let PlayState::Playing { playhead: old_playhead } = self.play_state {
                        self.old_pcm = Some(Shared::clone(self.pcm.as_ref().unwrap()));
                        self.fade_out_state = FadeOutState::Running {
                            old_playhead,
                            declick_gain: 1.0,
                            declick_frames_left: self.declick_frames,
                        };
                        self.play_state = PlayState::Playing { playhead: 0 };
                    } else if self.pcm.is_some() {
                        self.play_state = PlayState::Playing { playhead: 0 };
                    } else {
                        self.play_state = PlayState::Stopped;
                    }
                }
                */
                ProcessMsg::Stop => {
                    if let PlayState::Playing { playhead: old_playhead } = self.play_state {
                        self.old_pcm = Some(self.pcm.take().unwrap());

                        self.fade_out_state = FadeOutState::Running {
                            old_playhead,
                            current_gain: 1.0,
                            frames_left: self.crossfade_frames,
                        };

                        self.play_state = PlayState::Stopped;
                    }
                }
            }
        }
    }
}

impl PluginProcessor for SampleBrowserPlugProcessor {
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
        self.poll(in_events);

        let (mut buf_l, mut buf_r) = buffers.audio_out[0].stereo_f32_mut().unwrap();

        let buf_l_part = &mut buf_l[0..proc_info.frames];
        let buf_r_part = &mut buf_r[0..proc_info.frames];

        let mut apply_gain = false;

        if let PlayState::Playing { mut playhead } = self.play_state {
            let pcm = self.pcm.as_ref().unwrap();

            if playhead < pcm.len_frames() as usize {
                pcm.fill_stereo_f32(playhead, buf_l_part, buf_r_part);

                if self.fade_in_frames_left > 0 {
                    let fade_frames = proc_info.frames.min(self.fade_in_frames_left);

                    for i in 0..fade_frames {
                        self.fade_in_current_gain += self.crossfade_inc;

                        buf_l_part[i] *= self.fade_in_current_gain;
                        buf_r_part[i] *= self.fade_in_current_gain;
                    }

                    self.fade_in_frames_left -= fade_frames;
                }

                playhead += proc_info.frames;

                apply_gain = true;

                self.play_state = PlayState::Playing { playhead }
            } else {
                buf_l_part.fill(0.0);
                buf_r_part.fill(0.0);

                self.play_state = PlayState::Stopped;
            }
        } else {
            buf_l_part.fill(0.0);
            buf_r_part.fill(0.0);
        }

        if let FadeOutState::Running { mut old_playhead, mut current_gain, mut frames_left } =
            self.fade_out_state
        {
            let old_pcm = self.old_pcm.as_ref().unwrap();

            let mut running = true;

            let fade_out_buf_l_part = &mut self.fade_out_buf_l[0..proc_info.frames];
            let fade_out_buf_r_part = &mut self.fade_out_buf_r[0..proc_info.frames];

            if old_playhead < old_pcm.len_frames() as usize {
                old_pcm.fill_stereo_f32(old_playhead, fade_out_buf_l_part, fade_out_buf_r_part);

                old_playhead += proc_info.frames;

                apply_gain = true;
            } else {
                running = false;
            }

            if running {
                let fade_frames = proc_info.frames.min(frames_left);

                for i in 0..fade_frames {
                    current_gain -= self.crossfade_inc;

                    buf_l_part[i] += fade_out_buf_l_part[i] * current_gain;
                    buf_r_part[i] += fade_out_buf_r_part[i] * current_gain;
                }

                frames_left -= fade_frames;

                if frames_left == 0 {
                    running = false;
                }
            }

            if running {
                self.fade_out_state =
                    FadeOutState::Running { old_playhead, current_gain, frames_left }
            } else {
                self.fade_out_state = FadeOutState::Stopped;
                self.old_pcm = None;
            }
        }

        if apply_gain {
            let gain = self.params.gain.smoothed(proc_info.frames);
            if gain.is_smoothing() {
                debug_assert!(gain.values.len() >= proc_info.frames);

                for i in 0..proc_info.frames {
                    buf_l_part[i] *= gain.values[i];
                    buf_r_part[i] *= gain.values[i];
                }
            } else if gain[0].abs() >= std::f32::EPSILON {
                let g = gain[0];

                for i in 0..proc_info.frames {
                    buf_l_part[i] *= g;
                    buf_r_part[i] *= g;
                }
            }
        }

        if let PlayState::Stopped = &self.play_state {
            if let FadeOutState::Stopped = &self.fade_out_state {
                return ProcessStatus::Sleep;
            }
        }

        ProcessStatus::Continue
    }

    fn param_flush(&mut self, in_events: &EventBuffer, _out_events: &mut EventBuffer) {
        self.poll(in_events);
    }
}
