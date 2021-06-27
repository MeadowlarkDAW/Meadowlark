use basedrop::{Shared, SharedCell};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::shared_state::SharedState;

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
pub fn run_with_default_output(shared_state: Shared<SharedCell<SharedState>>) -> cpal::Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into(), shared_state),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into(), shared_state),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into(), shared_state),
    }
}

pub fn run<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    shared_state: Shared<SharedCell<SharedState>>,
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

                let mut shared = shared_state.get();

                // Should not panic because the non-rt thread always clones its shared state
                // before modifying it.
                let state = Shared::get_mut(&mut shared).unwrap();

                // Where the magic happens!
                state.process(n_frames);

                // Write first two audio buffers to device output
                for i in 0..n_frames {
                    // Safe because the scheduler ensures that all buffers have the length `n_frames`.
                    //
                    // TODO: Find a more ergonomic way to do this using a safe wrapper around a
                    // custom type? We also want to make it so a buffer can never be resized except
                    // by this scheduler at the top of this loop.
                    unsafe {
                        data[(i * 2)] = cpal::Sample::from::<f32>(
                            state.schedule.master_out_buffers[0].get_unchecked(i),
                        );
                        data[(i * 2) + 1] = cpal::Sample::from::<f32>(
                            state.schedule.master_out_buffers[1].get_unchecked(i),
                        );
                    }
                }
            },
            err_fn,
        )
        .unwrap();

    stream.play().unwrap();

    stream
}
