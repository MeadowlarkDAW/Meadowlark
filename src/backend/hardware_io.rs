use cpal::traits::{DeviceTrait, HostTrait};
use rusty_daw_time::SampleRate;

// This function is temporary. Eventually we should use rusty-daw-io instead.
pub fn default_sample_rate() -> Result<SampleRate, ()> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| ())?;
    let config = device.default_output_config().map_err(|_| ())?;

    Ok(SampleRate::new(config.sample_rate().0 as f64))
}
