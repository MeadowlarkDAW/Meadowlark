use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::frontend::{AudioGraph, BackendState};

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn run_with_default_output(state: BackendState) -> cpal::Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), state),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), state),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), state),
    }
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mut state: BackendState,
) -> cpal::Stream
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    assert_eq!(channels, 2); // Only support stereo output for test.

    /*
    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };
    */

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let max_buffer_size = if let cpal::BufferSize::Fixed(size) = config.buffer_size {
        size as usize
    } else {
        8096
    };
    let mut audio_buffers = Vec::<Vec<f32>>::new();
    audio_buffers.push(Vec::<f32>::with_capacity(max_buffer_size));
    audio_buffers.push(Vec::<f32>::with_capacity(max_buffer_size));

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                state.sync();

                let n_frames = data.len() / 2; // Assume output is stereo for test

                // Clear and resize all audio buffers
                for buffer in audio_buffers.iter_mut() {
                    buffer.clear();
                    buffer.resize(n_frames, 0.0);
                }

                // Write first two audio buffers to device output
                for i in 0..n_frames {
                    data[(i * 2)] = cpal::Sample::from::<f32>(&audio_buffers[0][i]);
                    data[(i * 2) + 1] = cpal::Sample::from::<f32>(&audio_buffers[1][i]);
                }
            },
            err_fn,
        )
        .unwrap();
    stream.play().unwrap();

    stream
}
