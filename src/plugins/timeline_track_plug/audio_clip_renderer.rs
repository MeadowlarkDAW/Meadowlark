use basedrop::Shared;
use meadowlark_plugin_api::decibel::db_to_coeff_f32;
use pcm_loader::PcmRAM;

use crate::resource::ResourceLoader;
use crate::state_system::source_state::project_track_state::CrossfadeType;
use crate::state_system::source_state::{AudioClipCopyableState, AudioClipState};
use crate::state_system::time::{FrameTime, TempoMap, Timestamp};

#[derive(Clone)]
pub struct AudioClipRenderer {
    pub pcm: Shared<PcmRAM>,

    pub(super) copyable: AudioClipRendererCopyable,
}

#[derive(Clone, Copy)]
pub(super) struct AudioClipRendererCopyable {
    pub timeline_start: FrameTime,
    pub timeline_end: FrameTime,

    pub clip_to_pcm_offset: i64,
    pub clip_length: FrameTime,

    // TODO: Automated gain.
    pub gain_amplitude: f32,

    pub incrossfade_type: CrossfadeType,
    pub incrossfade_len: u32,
    pub incrossfade_len_recip: f64,

    pub outcrossfade_type: CrossfadeType,
    pub outcrossfade_len: u32,
    pub outcrossfade_len_recip: f64,
}

impl AudioClipRendererCopyable {
    fn new(clip_state: &AudioClipCopyableState, tempo_map: &TempoMap) -> Self {
        let timeline_start = match clip_state.timeline_start {
            Timestamp::Musical(t) => tempo_map.musical_to_nearest_frame_round(t),
            Timestamp::Superclock(t) => t.to_nearest_frame_round(tempo_map.sample_rate()),
        };

        let timeline_end =
            timeline_start + clip_state.clip_length.to_nearest_frame_round(tempo_map.sample_rate());

        let mut clip_to_pcm_offset =
            clip_state.clip_to_pcm_offset.to_nearest_frame_round(tempo_map.sample_rate()).0 as i64;
        if clip_state.clip_to_pcm_offset_is_negative {
            clip_to_pcm_offset *= -1;
        }

        let incrossfade_len =
            clip_state.incrossfade_time.to_nearest_frame_round(tempo_map.sample_rate()).0 as u32;
        let outcrossfade_len =
            clip_state.outcrossfade_time.to_nearest_frame_round(tempo_map.sample_rate()).0 as u32;

        let incrossfade_len_recip =
            if incrossfade_len == 0 { 0.0 } else { 1.0 / incrossfade_len as f64 };
        let outcrossfade_len_recip =
            if outcrossfade_len == 0 { 0.0 } else { 1.0 / outcrossfade_len as f64 };

        let gain_amplitude = db_to_coeff_f32(clip_state.gain_db);

        Self {
            timeline_start,
            timeline_end,
            clip_to_pcm_offset,
            clip_length: timeline_end - timeline_start,
            gain_amplitude,
            incrossfade_type: clip_state.incrossfade_type,
            incrossfade_len,
            incrossfade_len_recip,
            outcrossfade_type: clip_state.outcrossfade_type,
            outcrossfade_len,
            outcrossfade_len_recip,
        }
    }
}

impl AudioClipRenderer {
    pub fn new(
        state: &AudioClipState,
        tempo_map: &TempoMap,
        resource_loader: &mut ResourceLoader,
    ) -> Self {
        let copyable = AudioClipRendererCopyable::new(&state.copyable, tempo_map);

        let (pcm, _result) = resource_loader.load_pcm(&state.pcm_key);

        Self { pcm, copyable }
    }

    pub fn sync_with_new_copyable_state(
        &mut self,
        state: &AudioClipCopyableState,
        tempo_map: &TempoMap,
    ) {
        self.copyable = AudioClipRendererCopyable::new(state, tempo_map);
    }

    pub fn timeline_start(&self) -> FrameTime {
        self.copyable.timeline_start
    }
    pub fn timeline_end(&self) -> FrameTime {
        self.copyable.timeline_end
    }

    pub fn render_channel(&self, frame: i64, out: &mut [f32], channel: usize) -> Result<bool, ()> {
        if channel >= self.pcm.channels() {
            return Err(());
        }

        let out_len = out.len();

        match self.calc_render_range(frame, out_len) {
            RenderRangeResult::OutOfRange => {
                // Out of range of clip and/or PCM data, so just fill the output buffer
                // with zeros.
                out.fill(0.0);
                return Ok(false);
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
                pcm_frames,
                // The frame in the PCM data at the frame `pcm_start_in_out_buf`.
                frame_in_pcm,
                /// The current position of the in crossfade, in the range
                /// `[0, self.incrossfade_len)`.
                incrossfade_pos,
                // The number of frames in the output buffer to apply the "in
                // crossfade" (starting from `clip_start_in_out_buf`).
                incrossfade_frames,
                // The frame in the output buffer where the "out crossfade"
                // starts.
                outcrossfade_start_in_out_buf,
                /// The current position of the out crossfade, in the range
                /// `[0, self.outcrossfade_len)`.
                outcrossfade_pos,
                // The number of frames in the output buffer to apply the "out
                // crossfade" (starting from `outcrossfade_start_in_out_buf`).
                outcrossfade_frames,
            } => {
                if pcm_start_in_out_buf > 0 {
                    // Clear the portion that is out-of range of the PCM data with
                    // zeros.
                    out[0..pcm_start_in_out_buf].fill(0.0);
                }

                if pcm_start_in_out_buf + pcm_frames < out_len {
                    // Clear the portion that is out-of range of the PCM data with
                    // zeros.
                    out[pcm_start_in_out_buf + pcm_frames..out_len].fill(0.0);
                }

                self.pcm
                    .fill_channel_f32(
                        channel,
                        frame_in_pcm as usize,
                        &mut out[pcm_start_in_out_buf..pcm_start_in_out_buf + pcm_frames],
                    )
                    .unwrap();

                if incrossfade_frames > 0 {
                    match self.copyable.incrossfade_type {
                        CrossfadeType::ConstantPower => {
                            // TODO
                        }
                        CrossfadeType::Linear => {
                            let out_part = &mut out
                                [clip_start_in_out_buf..clip_start_in_out_buf + incrossfade_frames];

                            let mut crossfade_pos = f64::from(incrossfade_pos);

                            for i in 0..incrossfade_frames {
                                let gain =
                                    (crossfade_pos * self.copyable.incrossfade_len_recip) as f32;

                                out_part[i] *= gain;
                                crossfade_pos += 1.0;
                            }
                        }
                    }
                }

                if outcrossfade_frames > 0 {
                    match self.copyable.outcrossfade_type {
                        CrossfadeType::ConstantPower => {
                            // TODO
                        }
                        CrossfadeType::Linear => {
                            let out_part = &mut out[outcrossfade_start_in_out_buf
                                ..outcrossfade_start_in_out_buf + outcrossfade_frames];

                            let mut crossfade_pos = f64::from(outcrossfade_pos);
                            let crossfade_len = f64::from(self.copyable.outcrossfade_len);

                            for i in 0..outcrossfade_frames {
                                let gain = ((crossfade_len - crossfade_pos)
                                    * self.copyable.outcrossfade_len_recip)
                                    as f32;

                                out_part[i] *= gain;
                                crossfade_pos += 1.0;
                            }
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    pub fn render_stereo(&self, frame: i64, out_left: &mut [f32], out_right: &mut [f32]) -> bool {
        let out_len = out_left.len().min(out_right.len());

        match self.calc_render_range(frame, out_len) {
            RenderRangeResult::OutOfRange => {
                // Out of range of clip and/or PCM data, so just fill the output buffer
                // with zeros.
                out_left.fill(0.0);
                out_right.fill(0.0);
                false
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
                pcm_frames,
                // The frame in the PCM data at the frame `pcm_start_in_out_buf`.
                frame_in_pcm,
                /// The current position of the in crossfade, in the range
                /// `[0, self.incrossfade_len)`.
                incrossfade_pos,
                // The number of frames in the output buffer to apply the "in
                // crossfade" (starting from `clip_start_in_out_buf`).
                incrossfade_frames,
                // The frame in the output buffer where the "out crossfade"
                // starts.
                outcrossfade_start_in_out_buf,
                /// The current position of the out crossfade, in the range
                /// `[0, self.outcrossfade_len)`.
                outcrossfade_pos,
                // The number of frames in the output buffer to apply the "out
                // crossfade" (starting from `outcrossfade_start_in_out_buf`).
                outcrossfade_frames,
            } => {
                if pcm_start_in_out_buf > 0 {
                    // Clear the portion that is out-of range of the PCM data with
                    // zeros.
                    out_left[0..pcm_start_in_out_buf].fill(0.0);
                    out_right[0..pcm_start_in_out_buf].fill(0.0);
                }

                if pcm_start_in_out_buf + pcm_frames < out_len {
                    // Clear the portion that is out-of range of the PCM data with
                    // zeros.
                    out_left[pcm_start_in_out_buf + pcm_frames..out_len].fill(0.0);
                    out_right[pcm_start_in_out_buf + pcm_frames..out_len].fill(0.0);
                }

                self.pcm.fill_stereo_f32(
                    frame_in_pcm as usize,
                    &mut out_left[pcm_start_in_out_buf..pcm_start_in_out_buf + pcm_frames],
                    &mut out_right[pcm_start_in_out_buf..pcm_start_in_out_buf + pcm_frames],
                );

                if incrossfade_frames > 0 {
                    match self.copyable.incrossfade_type {
                        CrossfadeType::ConstantPower => {
                            // TODO
                        }
                        CrossfadeType::Linear => {
                            let out_left_part = &mut out_left
                                [clip_start_in_out_buf..clip_start_in_out_buf + incrossfade_frames];
                            let out_right_part = &mut out_right
                                [clip_start_in_out_buf..clip_start_in_out_buf + incrossfade_frames];

                            let mut crossfade_pos = f64::from(incrossfade_pos);

                            for i in 0..incrossfade_frames {
                                let gain =
                                    (crossfade_pos * self.copyable.incrossfade_len_recip) as f32;

                                out_left_part[i] *= gain;
                                out_right_part[i] *= gain;

                                crossfade_pos += 1.0;
                            }
                        }
                    }
                }

                if outcrossfade_frames > 0 {
                    match self.copyable.outcrossfade_type {
                        CrossfadeType::ConstantPower => {
                            // TODO
                        }
                        CrossfadeType::Linear => {
                            let out_left_part = &mut out_left[outcrossfade_start_in_out_buf
                                ..outcrossfade_start_in_out_buf + outcrossfade_frames];
                            let out_right_part = &mut out_right[outcrossfade_start_in_out_buf
                                ..outcrossfade_start_in_out_buf + outcrossfade_frames];

                            let mut crossfade_pos = f64::from(outcrossfade_pos);
                            let crossfade_len = f64::from(self.copyable.outcrossfade_len);

                            for i in 0..outcrossfade_frames {
                                let gain = ((crossfade_len - crossfade_pos)
                                    * self.copyable.outcrossfade_len_recip)
                                    as f32;

                                out_left_part[i] *= gain;
                                out_right_part[i] *= gain;

                                crossfade_pos += 1.0;
                            }
                        }
                    }
                }

                true
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

        // The current position of the in crossfade, in the range
        // `[0, self.incrossfade_len)`.
        let mut incrossfade_pos = 0;

        // The number of frames in the output buffer to apply the "in
        // crossfade" (starting from `clip_start_in_out_buf`).
        let mut incrossfade_frames = 0;

        // The frame in the output buffer where the "out crossfade"
        // starts.
        let mut outcrossfade_start_in_out_buf = 0;

        // The current position of the out crossfade, in the range
        // `[0, self.outcrossfade_len)`.
        let mut outcrossfade_pos = 0;

        // The number of frames in the output buffer to apply the "out
        // crossfade" (starting from `outcrossfade_start_in_out_buf`).
        let mut outcrossfade_frames = 0;

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

        if frame_in_clip as u64 + clip_frames as u64 > self.copyable.clip_length.0 {
            if frame_in_clip as u64 >= self.copyable.clip_length.0 {
                // Out of range of clip. Fill with zeros.
                return RenderRangeResult::OutOfRange;
            }

            // Only copy the PCM samples up to the end of the clip.
            clip_frames = (self.copyable.clip_length.0 - frame_in_clip as u64) as usize;
        }

        pcm_frames = clip_frames;

        let mut frame_in_pcm_i64 = frame_in_clip as i64 + self.copyable.clip_to_pcm_offset;
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

        if self.copyable.incrossfade_len > 0 {
            if frame_in_clip < u64::from(self.copyable.incrossfade_len) {
                // Apply the start crossfade

                let fade_frames_left =
                    (u64::from(self.copyable.incrossfade_len) - frame_in_clip) as u32;

                incrossfade_pos = self.copyable.incrossfade_len - fade_frames_left;

                incrossfade_frames = (fade_frames_left as usize).min(clip_frames);
            }
        }

        if self.copyable.outcrossfade_len > 0 {
            if frame_in_clip + clip_frames as u64
                > self.copyable.clip_length.0 - (u64::from(self.copyable.outcrossfade_len))
            {
                // Apply the end crossfade

                let outcrossfade_start_offset = if frame_in_clip
                    >= self.copyable.clip_length.0 - (u64::from(self.copyable.outcrossfade_len))
                {
                    0
                } else {
                    ((self.copyable.clip_length.0 - (u64::from(self.copyable.outcrossfade_len)))
                        - frame_in_clip) as usize
                };

                let fade_frames_left = (self.copyable.clip_length.0 - frame_in_clip) as usize
                    - outcrossfade_start_offset;

                outcrossfade_start_in_out_buf = clip_start_in_out_buf + outcrossfade_start_offset;

                outcrossfade_pos =
                    (self.copyable.outcrossfade_len as usize - fade_frames_left) as u32;

                outcrossfade_frames =
                    (fade_frames_left as usize).min(clip_frames - outcrossfade_start_offset);
            }
        }

        RenderRangeResult::WithinRange {
            clip_start_in_out_buf,
            pcm_start_in_out_buf,
            pcm_frames,
            frame_in_pcm,
            incrossfade_pos,
            incrossfade_frames,
            outcrossfade_start_in_out_buf,
            outcrossfade_pos,
            outcrossfade_frames,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

        /// The current position of the in crossfade, in the range
        /// `[0, self.incrossfade_len)`.
        incrossfade_pos: u32,

        /// The number of frames in the output buffer to apply the "in
        /// crossfade" (starting from `clip_start_in_out_buf`).
        incrossfade_frames: usize,

        /// The frame in the output buffer where the "out crossfade"
        /// starts.
        outcrossfade_start_in_out_buf: usize,

        /// The current position of the out crossfade, in the range
        /// `[0, self.outcrossfade_len)`.
        outcrossfade_pos: u32,

        /// The number of frames in the output buffer to apply the "out
        /// crossfade" (starting from `outcrossfade_start_in_out_buf`).
        outcrossfade_frames: usize,
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

            copyable: AudioClipRendererCopyable {
                timeline_start: FrameTime(0),
                timeline_end: FrameTime(8),

                clip_to_pcm_offset: 0,
                clip_length: FrameTime(8),

                gain_amplitude: 1.0,

                incrossfade_type: CrossfadeType::Linear,
                incrossfade_len: 4,
                incrossfade_len_recip: 1.0 / 4.0,

                outcrossfade_type: CrossfadeType::Linear,
                outcrossfade_len: 3,
                outcrossfade_len_recip: 1.0 / 3.0,
            },
        };

        assert_eq!(&test_clip_renderer.calc_render_range(-8, 8), &RenderRangeResult::OutOfRange,);
        assert_eq!(&test_clip_renderer.calc_render_range(8, 8), &RenderRangeResult::OutOfRange,);

        assert_eq!(
            &test_clip_renderer.calc_render_range(0, 8),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 5,
                frame_in_pcm: 0,
                incrossfade_pos: 0,
                incrossfade_frames: 4,
                outcrossfade_start_in_out_buf: 5,
                outcrossfade_pos: 0,
                outcrossfade_frames: 3,
            },
        );

        assert_eq!(
            &test_clip_renderer.calc_render_range(1, 1),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 1,
                frame_in_pcm: 1,
                incrossfade_pos: 1,
                incrossfade_frames: 1,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_pos: 0,
                outcrossfade_frames: 0,
            },
        );

        assert_eq!(
            &test_clip_renderer.calc_render_range(-3, 4),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 3,
                pcm_start_in_out_buf: 3,
                pcm_frames: 1,
                frame_in_pcm: 0,
                incrossfade_pos: 0,
                incrossfade_frames: 1,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_pos: 0,
                outcrossfade_frames: 0,
            },
        );

        assert_eq!(&test_clip_renderer.calc_render_range(5, 4), &RenderRangeResult::OutOfRange,);

        assert_eq!(
            &test_clip_renderer.calc_render_range(3, 4),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 2,
                frame_in_pcm: 3,
                incrossfade_pos: 3,
                incrossfade_frames: 1,
                outcrossfade_start_in_out_buf: 2,
                outcrossfade_pos: 0,
                outcrossfade_frames: 2,
            },
        );

        test_clip_renderer.copyable.clip_length = FrameTime(4);

        assert_eq!(
            &test_clip_renderer.calc_render_range(-1, 6),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 4,
                frame_in_pcm: 0,
                incrossfade_pos: 0,
                incrossfade_frames: 4,
                outcrossfade_start_in_out_buf: 2,
                outcrossfade_pos: 0,
                outcrossfade_frames: 3,
            },
        );

        test_clip_renderer.copyable.clip_length = FrameTime(5);

        assert_eq!(
            &test_clip_renderer.calc_render_range(4, 5),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 1,
                frame_in_pcm: 4,
                incrossfade_pos: 0,
                incrossfade_frames: 0,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_pos: 2,
                outcrossfade_frames: 1,
            },
        );

        assert_eq!(
            &test_clip_renderer.calc_render_range(3, 5),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 0,
                pcm_start_in_out_buf: 0,
                pcm_frames: 2,
                frame_in_pcm: 3,
                incrossfade_pos: 3,
                incrossfade_frames: 1,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_pos: 1,
                outcrossfade_frames: 2,
            },
        );

        test_clip_renderer.copyable.clip_length = FrameTime(10);
        test_clip_renderer.copyable.clip_to_pcm_offset = 2;

        assert_eq!(
            &test_clip_renderer.calc_render_range(-1, 8),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 3,
                frame_in_pcm: 2,
                incrossfade_pos: 0,
                incrossfade_frames: 4,
                outcrossfade_start_in_out_buf: 0,
                outcrossfade_pos: 0,
                outcrossfade_frames: 0,
            },
        );

        assert_eq!(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 3,
                frame_in_pcm: 2,
                incrossfade_pos: 0,
                incrossfade_frames: 4,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_pos: 0,
                outcrossfade_frames: 1,
            },
        );

        test_clip_renderer.copyable.clip_to_pcm_offset = -2;

        assert_eq!(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 3,
                pcm_frames: 5,
                frame_in_pcm: 0,
                incrossfade_pos: 0,
                incrossfade_frames: 4,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_pos: 0,
                outcrossfade_frames: 1,
            },
        );

        test_clip_renderer.copyable.clip_to_pcm_offset = -7;

        assert_eq!(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 8,
                pcm_frames: 1,
                frame_in_pcm: 0,
                incrossfade_pos: 0,
                incrossfade_frames: 4,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_pos: 0,
                outcrossfade_frames: 1,
            },
        );

        test_clip_renderer.copyable.clip_to_pcm_offset = -8;

        assert_eq!(&test_clip_renderer.calc_render_range(-1, 9), &RenderRangeResult::OutOfRange,);

        test_clip_renderer.copyable.clip_to_pcm_offset = 4;

        assert_eq!(
            &test_clip_renderer.calc_render_range(-1, 9),
            &RenderRangeResult::WithinRange {
                clip_start_in_out_buf: 1,
                pcm_start_in_out_buf: 1,
                pcm_frames: 1,
                frame_in_pcm: 4,
                incrossfade_pos: 0,
                incrossfade_frames: 4,
                outcrossfade_start_in_out_buf: 8,
                outcrossfade_pos: 0,
                outcrossfade_frames: 1,
            },
        );

        test_clip_renderer.copyable.clip_to_pcm_offset = 5;

        assert_eq!(&test_clip_renderer.calc_render_range(-1, 9), &RenderRangeResult::OutOfRange,);
    }
}
