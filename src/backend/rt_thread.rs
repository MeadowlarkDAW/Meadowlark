use basedrop::{Shared, SharedCell};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::info;
use rusty_daw_audio_graph::AudioGraphExecutor;

use super::{GlobalNodeData, MAX_BLOCKSIZE};

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn run_with_default_output(
    executor: Shared<SharedCell<AudioGraphExecutor<GlobalNodeData, MAX_BLOCKSIZE>>>,
) -> Result<cpal::Stream, ()> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| ())?;
    let config = device.default_output_config().map_err(|_| ())?;

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), executor)?,
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), executor)?,
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), executor)?,
    };

    Ok(stream)
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    executor: Shared<SharedCell<AudioGraphExecutor<GlobalNodeData, MAX_BLOCKSIZE>>>,
) -> Result<cpal::Stream, ()>
where
    T: cpal::Sample,
{
    let channels = config.channels as usize;
    assert_eq!(channels, 2); // Only support stereo output for test.

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                // Where the magic happens!
                executor.get().process(data, |mut global_node_data, frames| {
                    global_node_data.transport.process(frames);
                });
            },
            err_fn,
        )
        .map_err(|_| ())?;

    stream.play().map_err(|_| ())?;

    let block_size_info = match config.buffer_size {
        cpal::BufferSize::Default => String::from("variable"),
        cpal::BufferSize::Fixed(b) => format!("{}", b),
    };

    info!(
        "opened audio stream | samplerate: {} | block_size: {} | output channels: {}",
        config.sample_rate.0, block_size_info, config.channels
    );

    Ok(stream)
}
