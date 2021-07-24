use cpal::traits::{DeviceTrait, HostTrait};
use rusty_daw_time::SampleRate;

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn default_sample_rate() -> SampleRate {
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    SampleRate::new(config.sample_rate().0 as f64)
}
