use basedrop::Shared;
use meadowlark_core_types::time::{FrameTime, SuperclockTime};
use pcm_loader::PcmRAM;

use super::resource_loader::PcmKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossfadeType {
    ConstantPower,
    Linear,
    //Symmetric, // TODO
    //Fast, // TODO
    //Slow, // TODO
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
    start_crossfade_len: usize,
    start_crossfade_len_recip: f64,

    end_crossfade_type: CrossfadeType,
    end_crossfade_len: usize,
    end_crossfade_len_recip: f64,
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

        if self.start_crossfade_len > 0 {
            if clip_frame < self.start_crossfade_len as u64 {
                let fade_frames_left = (self.start_crossfade_len as u64 - clip_frame) as usize;
                let fade_normal_pos = (self.start_crossfade_len - fade_frames_left) as f64
                    * self.start_crossfade_len_recip;

                let fade_frames = (fade_frames_left as usize).min(out.len());

                match self.start_crossfade_type {
                    CrossfadeType::ConstantPower => {
                        // TODO
                    }
                    CrossfadeType::Linear => {
                        let mut current_gain = fade_normal_pos;
                        let inc = self.start_crossfade_len_recip;
                        for i in 0..fade_frames {
                            out[i] *= current_gain as f32;
                            current_gain += inc;
                        }
                    }
                }
            }
        }

        if self.end_crossfade_len > 0 {
            if clip_frame >= self.clip_len_samples.0 - (self.end_crossfade_len as u64) {
                // Apply the end crossfade.

                let fade_frames_left = (self.clip_len_samples.0 - clip_frame) as usize;
                let fade_normal_pos = (self.end_crossfade_len - fade_frames_left) as f64
                    * self.end_crossfade_len_recip;

                let fade_frames = (fade_frames_left as usize).min(out.len());

                match self.start_crossfade_type {
                    CrossfadeType::ConstantPower => {
                        // TODO
                    }
                    CrossfadeType::Linear => {
                        let mut current_gain = 1.0 - fade_normal_pos;
                        let inc = self.end_crossfade_len_recip;
                        for i in 0..fade_frames {
                            out[i] *= current_gain as f32;
                            current_gain -= inc;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn render_stereo(start_sample: isize, out_left: &mut [f32], out_right: &mut [f32]) {
        let out_len = out_left.len().min(out_right.len());
    }
}
