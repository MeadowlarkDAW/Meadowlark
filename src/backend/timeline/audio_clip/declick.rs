use rusty_daw_core::{Frames, ProcFrames, SampleRate, Seconds, SmoothF32, SmoothOutputF32};

use crate::backend::timeline::TimelineTransport;

use super::super::TempoMap;

pub static DEFAULT_AUDIO_CLIP_DECLICK_TIME: Seconds = Seconds(2.0 / 1_000.0);

// TODO: Create crossfade automation manually instead of using the `Smooth` struct. Users
// will expect crossfades (especially loop crossfades) to sound exactly the same every
// time, so we should use an exact method instead of relying on the `Smooth` struct
// (which uses a filter internally).

/// Declicks audio clips when starting, stopping, seeking, or looping the timeline.
///
/// There exists only one `AudioClipDeclick` instance which is shared between all
/// `TimelineTrackNode`s.
pub struct AudioClipDeclick<const MAX_BLOCKSIZE: usize> {
    start_stop_fade: SmoothF32<MAX_BLOCKSIZE>,

    loop_crossfade_in: SmoothF32<MAX_BLOCKSIZE>,
    loop_crossfade_out: SmoothF32<MAX_BLOCKSIZE>,

    seek_crossfade_in: SmoothF32<MAX_BLOCKSIZE>,
    seek_crossfade_out: SmoothF32<MAX_BLOCKSIZE>,

    stop_fade_playhead: Option<Frames>,
    stop_fade_next_playhead: Frames,

    loop_crossfade_out_playhead: Frames,
    loop_crossfade_out_next_playhead: Frames,

    seek_crossfade_out_playhead: Frames,
    seek_crossfade_out_next_playhead: Frames,

    playing: bool,
    active: bool,
}

impl<const MAX_BLOCKSIZE: usize> AudioClipDeclick<MAX_BLOCKSIZE> {
    pub fn new(sample_rate: SampleRate) -> Self {
        let fade_time = DEFAULT_AUDIO_CLIP_DECLICK_TIME;

        let mut start_stop_fade = SmoothF32::<MAX_BLOCKSIZE>::new(0.0);
        start_stop_fade.set_speed(sample_rate, fade_time);

        let mut loop_crossfade_in = SmoothF32::<MAX_BLOCKSIZE>::new(0.0);
        loop_crossfade_in.set_speed(sample_rate, fade_time);

        let mut loop_crossfade_out = SmoothF32::<MAX_BLOCKSIZE>::new(1.0);
        loop_crossfade_out.set_speed(sample_rate, fade_time);

        let mut seek_crossfade_in = SmoothF32::<MAX_BLOCKSIZE>::new(0.0);
        seek_crossfade_in.set_speed(sample_rate, fade_time);

        let mut seek_crossfade_out = SmoothF32::<MAX_BLOCKSIZE>::new(1.0);
        seek_crossfade_out.set_speed(sample_rate, fade_time);

        Self {
            start_stop_fade,

            loop_crossfade_in,
            loop_crossfade_out,

            seek_crossfade_in,
            seek_crossfade_out,

            stop_fade_playhead: None,
            stop_fade_next_playhead: Frames(0),

            loop_crossfade_out_playhead: Frames(0),
            loop_crossfade_out_next_playhead: Frames(0),

            seek_crossfade_out_playhead: Frames(0),
            seek_crossfade_out_next_playhead: Frames(0),

            playing: false,
            active: false,
        }
    }

    pub fn update_tempo_map(&mut self, old_tempo_map: &TempoMap, new_tempo_map: &TempoMap) {
        if self.stop_fade_playhead.is_some() {
            let mt = old_tempo_map.frame_to_musical(self.stop_fade_next_playhead);
            self.stop_fade_next_playhead = new_tempo_map.musical_to_nearest_frame_round(mt);
        }

        if self.seek_crossfade_out.is_active() {
            let mt = old_tempo_map.frame_to_musical(self.seek_crossfade_out_next_playhead);
            self.seek_crossfade_out_next_playhead =
                new_tempo_map.musical_to_nearest_frame_round(mt);
        }

        if self.loop_crossfade_out.is_active() {
            let mt = old_tempo_map.frame_to_musical(self.loop_crossfade_out_next_playhead);
            self.loop_crossfade_out_next_playhead =
                new_tempo_map.musical_to_nearest_frame_round(mt);
        }
    }

    pub fn process(
        &mut self,
        proc_frames: ProcFrames<MAX_BLOCKSIZE>,
        timeline: &TimelineTransport<MAX_BLOCKSIZE>,
    ) {
        let frames = proc_frames.compiler_hint_frames();

        let mut just_stopped = false;

        if self.stop_fade_playhead.is_some() {
            if !self.start_stop_fade.is_active() {
                self.stop_fade_playhead = None;
            } else {
                self.stop_fade_playhead = Some(self.stop_fade_next_playhead);
                self.stop_fade_next_playhead += Frames::from_proc_frames(proc_frames);
            }
        }

        if self.playing != timeline.is_playing() {
            self.playing = timeline.is_playing();

            if self.playing {
                // Fade in.
                self.start_stop_fade.set(1.0);
            } else {
                // Fade out.
                self.start_stop_fade.set(0.0);
                just_stopped = true;

                self.stop_fade_playhead = Some(timeline.playhead());
                self.stop_fade_next_playhead =
                    timeline.playhead() + Frames::from_proc_frames(proc_frames);
            }
        }

        // Process the start/stop fades.
        self.start_stop_fade.process(proc_frames);
        self.start_stop_fade.update_status();

        // If the transport is not playing and did not just stop playing, then don't
        // start the seek crossfade. Otherwise a short sound could play when the user selects
        // the stop button when the transport is already stopped.
        let do_seek_crossfade = self.playing || just_stopped;

        if timeline.did_seek().is_some() && do_seek_crossfade {
            let seek_info = timeline.did_seek().unwrap();

            // Start the crossfade.

            self.seek_crossfade_in.reset(0.0);
            self.seek_crossfade_out.reset(1.0);

            self.seek_crossfade_in.set(1.0);
            self.seek_crossfade_out.set(0.0);

            self.seek_crossfade_in.process(proc_frames);
            self.seek_crossfade_in.update_status();

            self.seek_crossfade_out.process(proc_frames);
            self.loop_crossfade_out.update_status();

            self.seek_crossfade_out_playhead = seek_info.seeked_from_playhead;
            self.seek_crossfade_out_next_playhead =
                seek_info.seeked_from_playhead + Frames::from_proc_frames(proc_frames);
        } else {
            // Process any still-active seek crossfades.

            if self.seek_crossfade_out.is_active() {
                self.seek_crossfade_out_playhead = self.seek_crossfade_out_next_playhead;
                self.seek_crossfade_out_next_playhead += Frames::from_proc_frames(proc_frames);
            }

            self.seek_crossfade_in.process(proc_frames);
            self.seek_crossfade_in.update_status();

            self.seek_crossfade_out.process(proc_frames);
            self.seek_crossfade_out.update_status();
        }

        if let Some(loop_back) = timeline.do_loop_back() {
            let second_frames =
                ((loop_back.playhead_end - loop_back.loop_start).0 as usize).min(MAX_BLOCKSIZE);

            // Start the crossfade.

            self.loop_crossfade_in.reset(0.0);
            self.loop_crossfade_out.reset(1.0);

            self.loop_crossfade_in.set(1.0);
            self.loop_crossfade_out.set(0.0);

            if second_frames != 0 {
                self.loop_crossfade_in.process(second_frames.into());
                self.loop_crossfade_out.process(second_frames.into());

                self.loop_crossfade_in.update_status();
                self.loop_crossfade_out.update_status();
            }

            self.loop_crossfade_out_playhead = timeline.playhead();
            self.loop_crossfade_out_next_playhead =
                timeline.playhead() + Frames::from_proc_frames(proc_frames);
        } else {
            // Process any still-active loop crossfades.

            if self.loop_crossfade_out.is_active() {
                self.loop_crossfade_out_playhead = self.loop_crossfade_out_next_playhead;
                self.loop_crossfade_out_next_playhead += Frames::from_proc_frames(proc_frames);
            }

            self.loop_crossfade_in.process(proc_frames);
            self.loop_crossfade_in.update_status();

            self.loop_crossfade_out.process(proc_frames);
            self.loop_crossfade_out.update_status();
        }

        self.active = self.playing
            || self.start_stop_fade.is_active()
            || self.loop_crossfade_in.is_active()
            || self.loop_crossfade_out.is_active()
            || self.seek_crossfade_in.is_active()
            || self.seek_crossfade_out.is_active();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn stop_fade_playhead(&self) -> Option<Frames> {
        self.stop_fade_playhead
    }

    pub fn start_stop_fade(&self) -> SmoothOutputF32<MAX_BLOCKSIZE> {
        self.start_stop_fade.output()
    }

    pub fn loop_crossfade_in(&self) -> SmoothOutputF32<MAX_BLOCKSIZE> {
        self.loop_crossfade_in.output()
    }

    pub fn loop_crossfade_out(&self) -> (SmoothOutputF32<MAX_BLOCKSIZE>, Frames) {
        (self.loop_crossfade_out.output(), self.loop_crossfade_out_playhead)
    }

    pub fn seek_crossfade_in(&self) -> SmoothOutputF32<MAX_BLOCKSIZE> {
        self.seek_crossfade_in.output()
    }

    pub fn seek_crossfade_out(&self) -> (SmoothOutputF32<MAX_BLOCKSIZE>, Frames) {
        (self.seek_crossfade_out.output(), self.seek_crossfade_out_playhead)
    }
}
