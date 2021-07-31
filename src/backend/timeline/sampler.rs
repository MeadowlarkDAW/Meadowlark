use rusty_daw_time::{SampleRate, SampleTime, Seconds};

use crate::backend::graph_interface::StereoAudioBlockBuffer;
use crate::backend::resource_loader::{MonoPcm, StereoPcm};
use crate::backend::MAX_BLOCKSIZE;

// If the pcm sample rate matches the target sample rate, then no resampling will occur and samples will
// just be copied (with the start being rounded to the nearest sample).
pub fn sample_stereo(
    frames: usize,
    sample_rate: SampleRate,
    pcm: &StereoPcm,
    pcm_start: Seconds,
    out: &mut StereoAudioBlockBuffer,
    out_offset: usize,
) {
    // Make sure frames and out offset if valid.
    //
    // TODO: Handle this more transparently with a custom type.
    if out_offset >= MAX_BLOCKSIZE {
        // Out of range. Nothing to do.
        return;
    }
    let frames = frames.min(MAX_BLOCKSIZE - out_offset);

    // Calculate the range of samples from the pcm resource we need.

    let (pcm_start_smp, pcm_start_sub_smp) = pcm_start.to_sub_sample(pcm.sample_rate());
    let (pcm_frames, pcm_frames_sub_smp) = SampleTime(frames as i64)
        .to_seconds(sample_rate)
        .to_sub_sample(pcm.sample_rate());

    let mut zero_smps_before = 0;
    let mut zero_smps_after = 0;
    let mut pcm_frames = pcm_frames.0 as usize;

    let pcm_start_smp = if pcm_start_smp.0 < 0 {
        // Skip until the first sample in the pcm resource.
        zero_smps_before = (0 - pcm_start_smp.0) as usize;
        if zero_smps_before > pcm_frames {
            // Out of range. Nothing to do.
            return;
        }

        pcm_frames -= zero_smps_before;

        0
    } else {
        pcm_start_smp.0 as usize
    };

    if pcm_start_smp >= pcm.len() {
        // Out of range. Nothing to do.
        return;
    }

    if pcm_start_smp + pcm_frames >= pcm.len() {
        // Skip all samples after the last sample in the pcm resource.
        zero_smps_after = (pcm_start_smp + pcm_frames) - pcm.len();
        pcm_frames -= zero_smps_after;
    }

    let pcm_range_info = PcmRangeInfo {
        pcm_start_smp,
        pcm_start_sub_smp,
        pcm_frames,
        pcm_frames_sub_smp,
        zero_smps_before,
        zero_smps_after,
    };

    if pcm.sample_rate() == sample_rate {
        // No need to resample, just copy.
        copy_samples(pcm, &pcm_range_info, out, out_offset);
    } else {
        // Resample to project sample rate.

        // TODO
    }
}

struct PcmRangeInfo {
    /// Where to start reading from in the pcm resource.
    pcm_start_smp: usize,

    /// Additional sub-sample amount to `pcm_start_smp` (from `[0.0, 1.0)`).
    pcm_start_sub_smp: f64,

    /// How many frames to read from the pcm resource.
    pcm_frames: usize,

    /// Additional sub-sample amount to `pcm_frames` (from `[0.0, 1.0)`).
    pcm_frames_sub_smp: f64,

    /// How many zero samples to add before reading the pcm samples.
    zero_smps_before: usize,

    /// How many zero samples to add after reading the pcm samples.
    zero_smps_after: usize,
}

fn copy_samples(
    pcm: &StereoPcm,
    pcm_range_info: &PcmRangeInfo,
    out: &mut StereoAudioBlockBuffer,
    mut out_offset: usize,
) {
    let (pcm_start, pcm_frames, zero_smps_before) = if pcm_range_info.pcm_start_sub_smp >= 0.5 {
        // Round up to next sample.
        let pcm_start = pcm_range_info.pcm_start_smp + 1;

        // Check that we are still in bounds.
        let pcm_frames = if pcm_start + pcm_range_info.pcm_frames >= pcm.len() {
            pcm.len() - pcm_start
        } else {
            pcm_range_info.pcm_frames
        };

        let zero_smps_before = if pcm_range_info.zero_smps_before > 0 {
            pcm_range_info.zero_smps_before - 1
        } else {
            pcm_range_info.zero_smps_before
        };

        (pcm_start, pcm_frames, zero_smps_before)
    } else {
        (
            pcm_range_info.pcm_start_smp,
            pcm_range_info.pcm_frames,
            pcm_range_info.zero_smps_before,
        )
    };

    out_offset += zero_smps_before;

    if pcm_frames > 0 {
        &mut out.left[out_offset..out_offset + pcm_frames]
            .copy_from_slice(&pcm.left()[pcm_start..pcm_start + pcm_frames]);
        &mut out.right[out_offset..out_offset + pcm_frames]
            .copy_from_slice(&pcm.right()[pcm_start..pcm_start + pcm_frames]);
    }
}
