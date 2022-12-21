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

    incrossfade_type: CrossfadeType,
    start_crossfade_time: SuperclockTime,

    outcrossfade_type: CrossfadeType,
    end_crossfade_time: SuperclockTime,
}

impl AudioClipState {}

pub struct AudioClipRenderer {
    pcm: Shared<PcmRAM>,

    clip_to_pcm_offset: i64,
    clip_length: FrameTime,

    // TODO: Automated gain.
    gain_amplitude: f32,

    incrossfade_type: CrossfadeType,
    incrossfade_len: usize,
    incrossfade_len_recip: f64,

    outcrossfade_type: CrossfadeType,
    outcrossfade_len: usize,
    outcrossfade_len_recip: f64,
}

impl AudioClipRenderer {
    pub fn render_channel(&self, frame: i64, out: &mut [f32], channel: usize) -> Result<(), ()> {
        if channel >= self.pcm.channels() {
            return Err(());
        }

        match self.calc_render_range(frame, out.len()) {
            RenderRangeResult::OutOfRange => {
                // Out of range of clip and/or PCM data, so just fill the output buffer
                // with zeros.
                out.fill(0.0);
            }
            RenderRangeResult::WithinRange {
                // The frame in the output buffer where the clip starts.
                //
                // This will always be 0 unless the given `frame` is less than
                // zero.
                clip_start_in_out_buf,
                // The frame in the output buffer where the PCM data starts.
                pcm_start_in_out_buf,
                // The number of frames to fill in with PCM data (starting from
                // `pcm_start_in_out_buf`).
                pcm_frames: usize,
                // The frame in the PCM data at the frame `pcm_start_in_out_buf`.
                frame_in_pcm: u64,
                // The number of frames in the output buffer to apply the "in
                // crossfade" (starting from `clip_start_in_out_buf`).
                incrossfade_frames,
                // The normalized position of the "in crossfade" at the frame
                // `clip_start_in_out_buf`.
                incrossfade_normal_pos,
                // The frame in the output buffer where the "out crossfade"
                // starts.
                outcrossfade_start_in_out_buf,
                // The number of frames in the output buffer to apply the "out
                // crossfade" (starting from `outcrossfade_start_in_out_buf`).
                outcrossfade_frames,
                // The normalized position of the "out crossfade" at the frame
                // `outcrossfade_start_in_out_buf`.
                outcrossfade_normal_pos,
            } => {
                if pcm_start_in_out_buf > 0 {
                    // Clear the portion that is out-of range of the PCM data with
                    // zeros.
                    out[0..pcm_start_in_out_buf].fill(0.0);
                }

                /*
                match self.incrossfade_type {
                    CrossfadeType::ConstantPower => {
                        // TODO
                    }
                    CrossfadeType::Linear => {
                        let mut current_gain = fade_normal_pos;
                        let inc = self.incrossfade_len_recip;
                        for i in 0..fade_frames {
                            out[i] *= current_gain as f32;
                            current_gain += inc;
                        }
                    }
                }
                */
            }
        }

        Ok(())
    }

    pub fn render_stereo(&self, frame: i64, out_left: &mut [f32], out_right: &mut [f32]) {
        let out_len = out_left.len().min(out_right.len());

        match self.calc_render_range(frame, out_len) {
            RenderRangeResult::OutOfRange => {
                // Out of range of clip and/or PCM data, so just fill the output buffer
                // with zeros.
                out_left.fill(0.0);
                out_right.fill(0.0);
            }
            RenderRangeResult::WithinRange {
                // The frame in the output buffer where the clip starts.
                //
                // This will always be 0 unless the given `frame` is less than
                // zero.
                clip_start_in_out_buf,
                // The frame in the output buffer where the PCM data starts.
                pcm_start_in_out_buf,
                // The number of frames to fill in with PCM data (starting from
                // `pcm_start_in_out_buf`).
                pcm_frames: usize,
                // The frame in the PCM data at the frame `pcm_start_in_out_buf`.
                frame_in_pcm: u64,
                // The number of frames in the output buffer to apply the "in
                // crossfade" (starting from `clip_start_in_out_buf`).
                incrossfade_frames,
                // The normalized position of the "in crossfade" at the frame
                // `clip_start_in_out_buf`.
                incrossfade_normal_pos,
                // The frame in the output buffer where the "out crossfade"
                // starts.
                outcrossfade_start_in_out_buf,
                // The number of frames in the output buffer to apply the "out
                // crossfade" (starting from `outcrossfade_start_in_out_buf`).
                outcrossfade_frames,
                // The normalized position of the "out crossfade" at the frame
                // `outcrossfade_start_in_out_buf`.
                outcrossfade_normal_pos,
            } => {
                if pcm_start_in_out_buf > 0 {
                    // Clear the portion that is out-of range of the PCM data with
                    // zeros.
                    out_left[0..pcm_start_in_out_buf].fill(0.0);
                    out_right[0..pcm_start_in_out_buf].fill(0.0);
                }
            }
        }
    }

    fn calc_render_range(&self, frame: i64, out_len: usize) -> RenderRangeResult {
        // The frame in the output buffer where the clip starts.
        //
        // This will always be 0 unless the given `frame` is less than
        // zero.
        let mut clip_start_in_out_buf = 0;

        // The frame in the output buffer where the PCM data starts.
        let mut pcm_start_in_out_buf = 0;

        // The number of frames to fill in with PCM data (starting from
        // `pcm_start_in_out_buf`).
        let mut pcm_frames;

        // The frame in the PCM data at the frame `pcm_start_in_out_buf`.
        let frame_in_pcm: u64;

        // The number of frames in the output buffer to apply the "in
        // crossfade" (starting from `clip_start_in_out_buf`).
        let mut incrossfade_frames = 0;

        // The normalized position of the "in crossfade" at the frame
        // `clip_start_in_out_buf`.
        let mut incrossfade_normal_pos = 0.0;

        // The frame in the output buffer where the "out crossfade"
        // starts.
        let mut outcrossfade_start_in_out_buf = 0;

        // The number of frames in the output buffer to apply the "out
        // crossfade" (starting from `outcrossfade_start_in_out_buf`).
        let mut outcrossfade_frames = 0;

        // The normalized position of the "out crossfade" at the frame
        // `outcrossfade_start_in_out_buf`.
        let mut outcrossfade_normal_pos = 0.0;

        let mut frame_in_clip = frame;
        let mut clip_frames = out_len;

        if frame_in_clip < 0 {
            if frame_in_clip + clip_frames as i64 <= 0 {
                // Out of range of clip. Fill with zeros.
                return RenderRangeResult::OutOfRange;
            }

            // Clear all samples up to the start of the clip with zeros.
            clip_start_in_out_buf = -frame_in_clip as usize;
            pcm_start_in_out_buf = clip_start_in_out_buf;

            frame_in_clip = 0;
            clip_frames -= clip_start_in_out_buf;
        }

        if frame_in_clip as u64 + clip_frames as u64 > self.clip_length.0 {
            if frame_in_clip as u64 >= self.clip_length.0 {
                // Out of range of clip. Fill with zeros.
                return RenderRangeResult::OutOfRange;
            }

            // Only copy the PCM samples up to the end of the clip.
            clip_frames = (self.clip_length.0 - frame_in_clip as u64) as usize;
        }

        pcm_frames = clip_frames;

        let mut frame_in_pcm_i64 = frame_in_clip as i64 + self.clip_to_pcm_offset;
        if frame_in_pcm_i64 < 0 {
            if frame_in_pcm_i64 + pcm_frames as i64 <= 0
                || frame_in_pcm_i64 >= self.pcm.len_frames() as i64
            {
                // Out of range of PCM data. Fill with zeros.
                return RenderRangeResult::OutOfRange;
            }

            // Clear all samples up to the start of the PCM data with zeros.
            let pcm_zero_frames = -frame_in_pcm_i64 as usize;
            pcm_start_in_out_buf += pcm_zero_frames;

            frame_in_pcm_i64 = 0;
            pcm_frames -= pcm_zero_frames;
        }

        frame_in_pcm = frame_in_pcm_i64 as u64;

        if frame_in_pcm + pcm_frames as u64 > self.pcm.len_frames() as u64 {
            if frame_in_pcm >= self.pcm.len_frames() as u64 {
                // Out of range of PCM data. Fill with zeros.
                return RenderRangeResult::OutOfRange;
            }

            // Only copy the PCM samples up to the end of the PCM data.
            pcm_frames = (self.pcm.len_frames() as u64 - frame_in_pcm) as usize;
        }

        let frame_in_clip = frame_in_clip as u64;

        if self.incrossfade_len > 0 {
            if frame_in_clip < self.incrossfade_len as u64 {
                // Apply the start crossfade

                let fade_frames_left = (self.incrossfade_len as u64 - frame_in_clip) as usize;

                incrossfade_normal_pos =
                    (self.incrossfade_len - fade_frames_left) as f64 * self.incrossfade_len_recip;
                incrossfade_frames = (fade_frames_left as usize).min(clip_frames);
            }
        }

        if self.outcrossfade_len > 0 {
            if frame_in_clip + clip_frames as u64
                > self.clip_length.0 - (self.outcrossfade_len as u64)
            {
                // Apply the end crossfade

                let outcrossfade_start_offset = if frame_in_clip
                    >= self.clip_length.0 - (self.outcrossfade_len as u64)
                {
                    0
                } else {
                    ((self.clip_length.0 - (self.outcrossfade_len as u64)) - frame_in_clip) as usize
                };

                let fade_frames_left =
                    (self.clip_length.0 - frame_in_clip) as usize - outcrossfade_start_offset;

                outcrossfade_start_in_out_buf = clip_start_in_out_buf + outcrossfade_start_offset;

                outcrossfade_normal_pos =
                    (self.outcrossfade_len - fade_frames_left) as f64 * self.outcrossfade_len_recip;
                outcrossfade_frames =
                    (fade_frames_left as usize).min(clip_frames - outcrossfade_start_offset);
            }
        }

        RenderRangeResult::WithinRange {
            clip_start_in_out_buf,
            pcm_start_in_out_buf,
            pcm_frames,
            frame_in_pcm,
            incrossfade_frames,
            incrossfade_normal_pos,
            outcrossfade_start_in_out_buf,
            outcrossfade_frames,
            outcrossfade_normal_pos,
        }
    }
}

#[derive(Debug, Clone)]
enum RenderRangeResult {
    OutOfRange,
    WithinRange {
        /// The frame in the output buffer where the clip starts.
        ///
        /// This will always be 0 unless the given `frame` is less than
        /// zero.
        clip_start_in_out_buf: usize,

        /// The frame in the output buffer where the PCM data starts.
        pcm_start_in_out_buf: usize,

        /// The number of frames to fill in with PCM data (starting from
        /// `pcm_start_in_out_buf`).
        pcm_frames: usize,

        /// The frame in the PCM data at the frame `pcm_start_in_out_buf`.
        frame_in_pcm: u64,

        /// The number of frames in the output buffer to apply the "in
        /// crossfade" (starting from `clip_start_in_out_buf`).
        incrossfade_frames: usize,

        /// The normalized position of the "in crossfade" at the frame
        /// `clip_start_in_out_buf`.
        incrossfade_normal_pos: f64,

        /// The frame in the output buffer where the "out crossfade"
        /// starts.
        outcrossfade_start_in_out_buf: usize,

        /// The number of frames in the output buffer to apply the "out
        /// crossfade" (starting from `outcrossfade_start_in_out_buf`).
        outcrossfade_frames: usize,

        /// The normalized position of the "out crossfade" at the frame
        /// `outcrossfade_start_in_out_buf`.
        outcrossfade_normal_pos: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcm_loader::PcmRAMType;

    #[test]
    fn audio_clip_calc_render_range() {
        let collector = basedrop::Collector::new();

        let mut test_clip_renderer = AudioClipRenderer {
            pcm: Shared::new(
                &collector.handle(),
                PcmRAM::new(PcmRAMType::F32(vec![vec![1.0, 2.0, 3.0, 4.0, 5.0]]), 44100),
            ),

            clip_to_pcm_offset: 0,
            clip_length: FrameTime(8),

            gain_amplitude: 1.0,

            incrossfade_type: CrossfadeType::Linear,
            incrossfade_len: 4,
            incrossfade_len_recip: 1.0 / 4.0,

            outcrossfade_type: CrossfadeType::Linear,
            outcrossfade_len: 3,
            outcrossfade_len_recip: 1.0 / 3.0,
        };

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-8, 8),
            &RenderRangeResult::OutOfRange,
        );
        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(8, 8),
            &RenderRangeResult::OutOfRange,
        );

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(0, 8),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 5,
                frame_in_pcm: 0,
                incrossfade_frames: 4,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 5,
                outcrossfade_frames: 3,
                outcrossfade_normal_pos: 0.0,
            },
        );

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(1, 1),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 1,
                frame_in_pcm: 1,
                incrossfade_frames: 1,
                incrossfade_normal_pos: 0.25,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_frames: 0,
                outcrossfade_normal_pos: 0.0,
            },
        );

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-3, 4),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 3,
                pcm_start_in_out_buf: 3,
                pcm_frames: 1,
                frame_in_pcm: 0,
                incrossfade_frames: 1,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_frames: 0,
                outcrossfade_normal_pos: 0.0,
            },
        );

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(5, 4),
            &RenderRangeResult::OutOfRange,
        );

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(3, 4),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 2,
                frame_in_pcm: 3,
                incrossfade_frames: 1,
                incrossfade_normal_pos: 0.75,
                outcrossfade_start_in_out_buf: 2,
                outcrossfade_frames: 2,
                outcrossfade_normal_pos: 0.0,
            },
        );

        test_clip_renderer.clip_length = FrameTime(4);

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 6),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 4,
                frame_in_pcm: 0,
                incrossfade_frames: 4,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 2,
                outcrossfade_frames: 3,
                outcrossfade_normal_pos: 0.0,
            },
        );

        test_clip_renderer.clip_length = FrameTime(5);

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(4, 5),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 1,
                frame_in_pcm: 4,
                incrossfade_frames: 0,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_frames: 1,
                outcrossfade_normal_pos: 2.0 / 3.0,
            },
        );

        test_clip_renderer.clip_length = FrameTime(10);
        test_clip_renderer.clip_to_pcm_offset = 2;

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 8),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 3,
                frame_in_pcm: 2,
                incrossfade_frames: 4,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_frames: 0,
                outcrossfade_normal_pos: 0.0,
            },
        );

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 3,
                frame_in_pcm: 2,
                incrossfade_frames: 4,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_frames: 1,
                outcrossfade_normal_pos: 0.0,
            },
        );

        test_clip_renderer.clip_to_pcm_offset = -2;

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 3,
                pcm_frames: 5,
                frame_in_pcm: 0,
                incrossfade_frames: 4,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_frames: 1,
                outcrossfade_normal_pos: 0.0,
            },
        );

        test_clip_renderer.clip_to_pcm_offset = -7;

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 8,
                pcm_frames: 1,
                frame_in_pcm: 0,
                incrossfade_frames: 4,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_frames: 1,
                outcrossfade_normal_pos: 0.0,
            },
        );

        test_clip_renderer.clip_to_pcm_offset = -8;

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::OutOfRange,
        );

        test_clip_renderer.clip_to_pcm_offset = 4;

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 1,
                frame_in_pcm: 4,
                incrossfade_frames: 4,
                incrossfade_normal_pos: 0.0,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_frames: 1,
                outcrossfade_normal_pos: 0.0,
            },
        );

        test_clip_renderer.clip_to_pcm_offset = 5;

        assert_render_ranges_equal(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::OutOfRange,
        );
    }

    fn assert_render_ranges_equal(a: &RenderRangeResult, b: &RenderRangeResult) {
        if !render_ranges_equal(a, b) {
            panic!("render ranges not equal:\n a: {:?},\n b: {:?}", a, b);
        }
    }

    fn render_ranges_equal(a: &RenderRangeResult, b: &RenderRangeResult) -> bool {
        if let RenderRangeResult::WithinRange {
            clip_start_in_out_buf: a_clip_start_in_out_buf,
            pcm_start_in_out_buf: a_pcm_start_in_out_buf,
            pcm_frames: a_pcm_frames,
            frame_in_pcm: a_frame_in_pcm,
            incrossfade_frames: a_incrossfade_frames,
            incrossfade_normal_pos: a_incrossfade_normal_pos,
            outcrossfade_start_in_out_buf: a_outcrossfade_start_in_out_buf,
            outcrossfade_frames: a_outcrossfade_frames,
            outcrossfade_normal_pos: a_outcrossfade_normal_pos,
        } = a
        {
            if let RenderRangeResult::WithinRange {
                clip_start_in_out_buf: b_clip_start_in_out_buf,
                pcm_start_in_out_buf: b_pcm_start_in_out_buf,
                pcm_frames: b_pcm_frames,
                frame_in_pcm: b_frame_in_pcm,
                incrossfade_frames: b_incrossfade_frames,
                incrossfade_normal_pos: b_incrossfade_normal_pos,
                outcrossfade_start_in_out_buf: b_outcrossfade_start_in_out_buf,
                outcrossfade_frames: b_outcrossfade_frames,
                outcrossfade_normal_pos: b_outcrossfade_normal_pos,
            } = b
            {
                if a_clip_start_in_out_buf != b_clip_start_in_out_buf
                    || a_pcm_start_in_out_buf != b_pcm_start_in_out_buf
                    || a_pcm_frames != b_pcm_frames
                    || a_frame_in_pcm != b_frame_in_pcm
                    || a_incrossfade_frames != b_incrossfade_frames
                    || a_outcrossfade_start_in_out_buf != b_outcrossfade_start_in_out_buf
                    || a_outcrossfade_frames != b_outcrossfade_frames
                {
                    return false;
                }

                if (a_incrossfade_normal_pos - b_incrossfade_normal_pos).abs() > f64::EPSILON {
                    return false;
                }
                if (a_outcrossfade_normal_pos - b_outcrossfade_normal_pos).abs() > f64::EPSILON {
                    return false;
                }

                true
            } else {
                false
            }
        } else if let RenderRangeResult::WithinRange { .. } = b {
            false
        } else {
            true
        }
    }
}
