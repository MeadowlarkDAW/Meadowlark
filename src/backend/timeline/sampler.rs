use rusty_daw_time::{SampleRate, SampleTime, Seconds};

use crate::backend::graph_interface::StereoAudioBlockBuffer;
use crate::backend::resource_loader::{MonoPcm, StereoPcm};
use crate::backend::MAX_BLOCKSIZE;

// If the pcm sample rate matches the target sample rate, then no resampling will occur and samples will
// just be copied (with the start being rounded to the nearest sample).
//
// This will overwrite the samples in the given `out` buffer.
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

    let mut pcm_frames = pcm_frames.0 as usize;

    if pcm_start_smp.0 >= pcm.len() as i64 {
        // Out of range. Nothing to do.
        return;
    }

    let pcm_range_info = PcmRangeInfo {
        pcm_start_smp,
        pcm_start_sub_smp,
        pcm_frames,
        pcm_frames_sub_smp,
    };

    if pcm.sample_rate() == sample_rate {
        // No need to resample, just copy.
        copy_samples_nearest(pcm, &pcm_range_info, out, out_offset);
    } else {
        // Resample to project sample rate.

        // TODO
    }
}

struct PcmRangeInfo {
    /// Where to start reading from in the pcm resource (floored).
    pcm_start_smp: SampleTime,

    /// Additional sub-sample amount to `pcm_start_smp` (from `[0.0, 1.0)`).
    pcm_start_sub_smp: f64,

    /// How many frames to read from the pcm resource (floored).
    pcm_frames: usize,

    /// Additional sub-sample amount to `pcm_frames` (from `[0.0, 1.0)`).
    pcm_frames_sub_smp: f64,
}

fn copy_samples_nearest(
    pcm: &StereoPcm,
    pcm_range_info: &PcmRangeInfo,
    out: &mut StereoAudioBlockBuffer,
    mut out_offset: usize,
) {
    let pcm_start_smp = if pcm_range_info.pcm_start_sub_smp >= 0.5 {
        // Round up to next sample.
        pcm_range_info.pcm_start_smp + SampleTime(1)
    } else {
        pcm_range_info.pcm_start_smp
    };

    let (zeros_after, mut pcm_frames) = if pcm_start_smp.0 + pcm_range_info.pcm_frames as i64
        > pcm.len() as i64
    {
        let zeros_after = (pcm_start_smp.0 + pcm_range_info.pcm_frames as i64) as usize - pcm.len();
        let pcm_frames = pcm_range_info.pcm_frames - zeros_after;
        (zeros_after, pcm_frames)
    } else {
        (0, pcm_range_info.pcm_frames)
    };

    let (pcm_start, zeros_before) = if pcm_start_smp.0 < 0 {
        (0, (0 - pcm_start_smp.0) as usize)
    } else {
        (pcm_start_smp.0 as usize, 0)
    };

    pcm_frames -= zeros_before;

    if zeros_before > 0 {
        // Fill in zeros.
        &mut out.left[out_offset..out_offset + zeros_before].fill(0.0);
        &mut out.right[out_offset..out_offset + zeros_before].fill(0.0);
    }

    out_offset += zeros_before;

    if pcm_frames > 0 {
        &mut out.left[out_offset..out_offset + pcm_frames]
            .copy_from_slice(&pcm.left()[pcm_start..pcm_start + pcm_frames]);
        &mut out.right[out_offset..out_offset + pcm_frames]
            .copy_from_slice(&pcm.right()[pcm_start..pcm_start + pcm_frames]);
    }

    out_offset += pcm_frames;

    if zeros_after > 0 {
        // Fill in zeros.
        &mut out.left[out_offset..out_offset + zeros_after].fill(0.0);
        &mut out.right[out_offset..out_offset + zeros_after].fill(0.0);
    }
}
