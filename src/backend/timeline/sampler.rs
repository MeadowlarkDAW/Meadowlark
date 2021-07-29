use rusty_daw_time::{SampleRate, SampleTime, Seconds};

use crate::backend::graph_interface::StereoAudioBlockBuffer;
use crate::backend::resource_loader::{MonoPcm, StereoPcm};
use crate::backend::MAX_BLOCKSIZE;

pub fn sample_stereo(
    frames: usize,
    sample_rate: SampleRate,
    pcm: &StereoPcm,
    pcm_start: Seconds,
    out: &mut StereoAudioBlockBuffer,
    out_offset: usize,
) {
    if pcm.sample_rate() == sample_rate {
        // No need to resample, just copy.

        let pcm_start_smp = pcm_start.to_nearest_sample_round(pcm.sample_rate());
        let mut out_offset = out_offset;
        let mut len = frames;

        if pcm_start_smp.0 >= pcm.len() as i64 || pcm_start_smp.0 + len as i64 <= 0 {
            // Out of range, nothing to do.
            return;
        }

        let pcm_start = if pcm_start_smp.0 < 0 {
            // Skip until the first sample in the pcm resource.
            len -= (0 - pcm_start_smp.0) as usize;
            out_offset += (0 - pcm_start_smp.0) as usize;
            0
        } else {
            pcm_start_smp.0 as usize
        };

        if pcm_start + len > pcm.len() {
            // Stop after the last sample in the pcm resource.
            len = pcm.len() - pcm_start;
        }

        &mut out.left[out_offset..out_offset + len]
            .copy_from_slice(&pcm.left()[pcm_start..pcm_start + len]);
        &mut out.right[out_offset..out_offset + len]
            .copy_from_slice(&pcm.right()[pcm_start..pcm_start + len]);
    } else {
        // Resample to project sample rate.

        let sample_secs = SampleTime(frames as i64).to_seconds(pcm.sample_rate());

        let (pcm_start_smp, start_sub_sample) = pcm_start.to_sub_sample(pcm.sample_rate());
        let (pcm_end_smp, end_sub_sample) =
            (pcm_start + sample_secs).to_sub_sample(pcm.sample_rate());

        // TODO
    }
}
