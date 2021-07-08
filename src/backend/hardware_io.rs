use cpal::traits::{DeviceTrait, HostTrait};

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn default_sample_rate() -> f32 {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    config.sample_rate().0 as f32
}
