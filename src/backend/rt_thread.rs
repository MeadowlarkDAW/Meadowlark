use basedrop::{Shared, SharedCell};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::info;

use super::graph::CompiledGraph;

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn run_with_default_output(
    graph_state: Shared<SharedCell<CompiledGraph>>,
) -> Result<cpal::Stream, ()> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| ())?;
    let config = device.default_output_config().map_err(|_| ())?;

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), graph_state)?,
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), graph_state)?,
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), graph_state)?,
    };

    Ok(stream)
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    graph_state: Shared<SharedCell<CompiledGraph>>,
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
                graph_state.get().process(data);
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
