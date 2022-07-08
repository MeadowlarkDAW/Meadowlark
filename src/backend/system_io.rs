use std::error::Error;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Stream;
use dropseed::DSEngineAudioThread;
use meadowlark_core_types::SampleRate;
use rtrb::{Producer, RingBuffer};

const HANDLE_TO_STREAM_MSG_SIZE: usize = 8;

#[derive(Debug)]
enum HandleToStreamMsg {
    NewEngineAudioThread(DSEngineAudioThread),
    DropEngineAudioThread,
}

pub struct SystemIOStreamHandle {
    cpal_stream: Stream,
    to_stream_tx: Producer<HandleToStreamMsg>,
    sample_rate: SampleRate,
}

impl SystemIOStreamHandle {
    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn engine_activated(&mut self, engine_audio_thread: DSEngineAudioThread) {
        self.to_stream_tx
            .push(HandleToStreamMsg::NewEngineAudioThread(engine_audio_thread))
            .unwrap();
    }

    pub fn engine_deactivated(&mut self) {
        self.to_stream_tx.push(HandleToStreamMsg::DropEngineAudioThread).unwrap();
    }
}

/// This is temporary. Eventually we will have a more sophisticated and
/// configurable system using `rainout`.
pub fn temp_spawn_cpal_default_output_only() -> Result<SystemIOStreamHandle, Box<dyn Error>> {
    let (to_stream_tx, mut from_handle_rx) =
        RingBuffer::<HandleToStreamMsg>::new(HANDLE_TO_STREAM_MSG_SIZE);

    let cpal_host = cpal::default_host();

    let device = cpal_host
        .default_output_device()
        .ok_or("CPAL: no default audio out device found".to_string())?;
    
    log::info!("Selected default CPAL output device: {:?}", &device.name());

    let config = device.default_output_config()?;

    let num_out_channels = usize::from(config.channels());
    let sample_rate: SampleRate = config.sample_rate().0.into();

    let mut engine_audio_thread: Option<DSEngineAudioThread> = None;

    log::info!("Starting CPAL stream with config {:?}...", &config);

    let cpal_stream = device.build_output_stream(
        &config.into(),
        move |audio_buffer: &mut [f32], _: &cpal::OutputCallbackInfo| {
            while let Ok(msg) = from_handle_rx.pop() {
                match msg {
                    HandleToStreamMsg::NewEngineAudioThread(new_engine_audio_thread) => {
                        engine_audio_thread = Some(new_engine_audio_thread);
                    }
                    HandleToStreamMsg::DropEngineAudioThread => {
                        engine_audio_thread = None;
                    }
                }
            }

            if let Some(engine_audio_thread) = &mut engine_audio_thread {
                engine_audio_thread
                    .process_cpal_interleaved_output_only(num_out_channels, audio_buffer);
            }
        },
        |e| {
            // TODO: Better handling of the system IO stream crashing.
            panic!("{}", e);
        },
    )?;

    log::info!("Successfully started CPAL stream");

    Ok(SystemIOStreamHandle { cpal_stream, to_stream_tx, sample_rate })
}
