use basedrop::{Shared, SharedCell};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::graph_state::GraphState;

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn default_sample_rate_and_buffer_size() -> (f32, usize) {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    let buffer_size = match config.buffer_size() {
        cpal::SupportedBufferSize::Range { max, .. } => *max,
        _ => 8192,
    };

    (config.sample_rate().0 as f32, buffer_size as usize)
}

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn run_with_default_output(graph_state: Shared<SharedCell<GraphState>>) -> cpal::Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), graph_state),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), graph_state),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), graph_state),
    }
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    graph_state: Shared<SharedCell<GraphState>>,
) -> cpal::Stream
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
                let n_frames = data.len() / 2; // Assume output is stereo for test

                // Where the magic happens!
                graph_state.get().process(n_frames, data);
            },
            err_fn,
        )
        .unwrap();

    stream.play().unwrap();

    stream
}
