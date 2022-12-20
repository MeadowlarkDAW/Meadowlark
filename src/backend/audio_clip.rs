use meadowlark_core_types::time::{FrameTime, SuperclockTime};
use pcm_loader::PcmRAM;

use super::resource_loader::PcmKey;

use basedrop::{Collector, Shared};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossfadeType {
    ConstantPower,
    Linear,
    Symmetric,
    Fast,
    Slow,
}

impl Default for CrossfadeType {
    fn default() -> Self {
        CrossfadeType::ConstantPower
    }
}

pub struct AudioClipState {
    key: PcmKey,

    start_sample: SuperclockTime,
    length_samples: SuperclockTime,

    // TODO: Automated gain.
    gain_db: f32,

    start_crossfade_type: CrossfadeType,
    start_crossfade_time: SuperclockTime,

    end_crossfade_type: CrossfadeType,
    end_crossfade_time: SuperclockTime,
}

impl AudioClipState {}

pub struct AudioClipRenderer {
    pcm: Shared<PcmRAM>,

    start_sample_offset: i64,
    clip_len_samples: FrameTime,

    // TODO: Automated gain.
    gain_amplitude: f32,

    start_crossfade_type: CrossfadeType,
    start_crossfade_end_sample: FrameTime,

    end_crossfade_type: CrossfadeType,
    end_crossfade_start_sample: FrameTime,
}

impl AudioClipRenderer {
    pub fn render_channel(
        &self,
        frame: isize,
        mut out: &mut [f32],
        channel: usize,
    ) -> Result<(), ()> {
        if channel >= self.pcm.channels() {
            return Err(());
        }

        let mut clip_frame = frame;

        if clip_frame < 0 {
            if clip_frame + out.len() as isize <= 0 {
                // Out of range of clip. Fill with zeros.
                out.fill(0.0);
                return Ok(());
            }

            // Clear all samples up to the start of the clip with zeros.
            let zero_frames = clip_frame.abs() as usize;
            out[0..zero_frames].fill(0.0);

            clip_frame = 0;
            out = &mut out[zero_frames..];
        } else if clip_frame as u64 >= self.clip_len_samples.0 {
            // Out of range of clip. Fill with zeros.
            out.fill(0.0);
            return Ok(());
        }

        let mut pcm_frame = clip_frame as i64 + self.start_sample_offset;
        let mut pcm_zero_frames = 0;
        if pcm_frame < 0 {
            if pcm_frame + out.len() as i64 <= 0 {
                // Out of range of PCM data. Fill with zeros.
                out.fill(0.0);
                return Ok(());
            }

            // Clear all samples up to the start of the PCM data with zeros.
            pcm_zero_frames = pcm_frame.abs() as usize;
            out[0..pcm_zero_frames].fill(0.0);

            pcm_frame = 0;
        }

        let num_samples_filled = self
            .pcm
            .fill_channel_f32(channel, pcm_frame as usize, &mut out[pcm_zero_frames..])
            .unwrap();

        if num_samples_filled == 0 {
            // Out of range of PCM data. The `fill_channel_f32` method will
            // have already filled the buffer with zeros.
            return Ok(());
        }

        let clip_frame = clip_frame as u64;
        if clip_frame < self.start_crossfade_end_sample.0 {
            // Apply the start crossfade.
        }

        if clip_frame >= self.end_crossfade_start_sample.0 {
            // Apply the end crossfade.
        }

        Ok(())
    }

    pub fn render_stereo(start_sample: isize, out_left: &mut [f32], out_right: &mut [f32]) {
        let out_len = out_left.len().min(out_right.len());
    }
}
