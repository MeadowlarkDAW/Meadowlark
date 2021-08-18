use rusty_daw_time::{SampleRate, SampleTime, Seconds, TempoMap};

use crate::backend::audio_graph::ProcInfo;
use crate::backend::parameter::{Smooth, SmoothOutput};
use crate::backend::timeline::TimelineTransport;
use crate::backend::MAX_BLOCKSIZE;

pub static DEFAULT_AUDIO_CLIP_DECLICK_TIME: Seconds = Seconds(2.0 / 1_000.0);

/// Declicks audio clips when starting, stopping, seeking, or looping the timeline.
///
/// There exists only one `AudioClipDeclick` instance which is shared between all
/// `TimelineTrackNode`s.
pub struct AudioClipDeclick {
    start_stop_fade: Smooth<f32>,

    loop_crossfade_in: Smooth<f32>,
    loop_crossfade_out: Smooth<f32>,

    seek_crossfade_in: Smooth<f32>,
    seek_crossfade_out: Smooth<f32>,

    stop_fade_playhead: Option<SampleTime>,
    stop_fade_next_playhead: SampleTime,

    loop_crossfade_out_playhead: SampleTime,
    loop_crossfade_out_next_playhead: SampleTime,

    seek_crossfade_out_playhead: SampleTime,
    seek_crossfade_out_next_playhead: SampleTime,

    playing: bool,
    active: bool,
}

impl AudioClipDeclick {
    pub fn new(sample_rate: SampleRate) -> Self {
        let fade_time = DEFAULT_AUDIO_CLIP_DECLICK_TIME;

        let mut start_stop_fade = Smooth::<f32>::new(0.0);
        start_stop_fade.set_speed(sample_rate, fade_time);

        let mut loop_crossfade_in = Smooth::<f32>::new(0.0);
        loop_crossfade_in.set_speed(sample_rate, fade_time);

        let mut loop_crossfade_out = Smooth::<f32>::new(1.0);
        loop_crossfade_out.set_speed(sample_rate, fade_time);

        let mut seek_crossfade_in = Smooth::<f32>::new(0.0);
        seek_crossfade_in.set_speed(sample_rate, fade_time);

        let mut seek_crossfade_out = Smooth::<f32>::new(1.0);
        seek_crossfade_out.set_speed(sample_rate, fade_time);

        Self {
            start_stop_fade,

            loop_crossfade_in,
            loop_crossfade_out,

            seek_crossfade_in,
            seek_crossfade_out,

            stop_fade_playhead: None,
            stop_fade_next_playhead: SampleTime(0),

            loop_crossfade_out_playhead: SampleTime(0),
            loop_crossfade_out_next_playhead: SampleTime(0),

            seek_crossfade_out_playhead: SampleTime(0),
            seek_crossfade_out_next_playhead: SampleTime(0),

            playing: false,
            active: false,
        }
    }

    pub fn update_tempo_map(&mut self, old_tempo_map: &TempoMap, new_tempo_map: &TempoMap) {
        if self.stop_fade_playhead.is_some() {
            let mt = self.stop_fade_next_playhead.to_musical(old_tempo_map);
            self.stop_fade_next_playhead = mt.to_nearest_sample_round(new_tempo_map);
        }

        if self.seek_crossfade_out.is_active() {
            let mt = self
                .seek_crossfade_out_next_playhead
                .to_musical(old_tempo_map);
            self.seek_crossfade_out_next_playhead = mt.to_nearest_sample_round(new_tempo_map);
        }

        if self.loop_crossfade_out.is_active() {
            let mt = self
                .loop_crossfade_out_next_playhead
                .to_musical(old_tempo_map);
            self.loop_crossfade_out_next_playhead = mt.to_nearest_sample_round(new_tempo_map);
        }
    }

    pub fn process(&mut self, proc_info: &ProcInfo, timeline: &TimelineTransport) {
        let mut just_stopped = false;

        if self.stop_fade_playhead.is_some() {
            if !self.start_stop_fade.is_active() {
                self.stop_fade_playhead = None;
            } else {
                self.stop_fade_playhead = Some(self.stop_fade_next_playhead);
                self.stop_fade_next_playhead += SampleTime::from_usize(proc_info.frames());
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
                    timeline.playhead() + SampleTime::from_usize(proc_info.frames());
            }
        }

        // Process the start/stop fades.
        self.start_stop_fade.process(proc_info.frames());
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

            self.seek_crossfade_in.process(proc_info.frames());
            self.seek_crossfade_in.update_status();

            self.seek_crossfade_out.process(proc_info.frames());
            self.loop_crossfade_out.update_status();

            self.seek_crossfade_out_playhead = seek_info.seeked_from_playhead;
            self.seek_crossfade_out_next_playhead =
                seek_info.seeked_from_playhead + SampleTime::from_usize(proc_info.frames());
        } else {
            // Process any still-active seek crossfades.

            if self.seek_crossfade_out.is_active() {
                self.seek_crossfade_out_playhead = self.seek_crossfade_out_next_playhead;
                self.seek_crossfade_out_next_playhead += SampleTime::from_usize(proc_info.frames());
            }

            self.seek_crossfade_in.process(proc_info.frames());
            self.seek_crossfade_in.update_status();

            self.seek_crossfade_out.process(proc_info.frames());
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
                self.loop_crossfade_in.process(second_frames);
                self.loop_crossfade_out.process(second_frames);

                self.loop_crossfade_in.update_status();
                self.loop_crossfade_out.update_status();
            }

            self.loop_crossfade_out_playhead = timeline.playhead();
            self.loop_crossfade_out_next_playhead =
                timeline.playhead() + SampleTime::from_usize(proc_info.frames());
        } else {
            // Process any still-active loop crossfades.

            if self.loop_crossfade_out.is_active() {
                self.loop_crossfade_out_playhead = self.loop_crossfade_out_next_playhead;
                self.loop_crossfade_out_next_playhead += SampleTime::from_usize(proc_info.frames());
            }

            self.loop_crossfade_in.process(proc_info.frames());
            self.loop_crossfade_in.update_status();

            self.loop_crossfade_out.process(proc_info.frames());
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

    pub fn stop_fade_playhead(&self) -> Option<SampleTime> {
        self.stop_fade_playhead
    }

    pub fn start_stop_fade(&self) -> SmoothOutput<f32> {
        self.start_stop_fade.output()
    }

    pub fn loop_crossfade_in(&self) -> SmoothOutput<f32> {
        self.loop_crossfade_in.output()
    }

    pub fn loop_crossfade_out(&self) -> (SmoothOutput<f32>, SampleTime) {
        (
            self.loop_crossfade_out.output(),
            self.loop_crossfade_out_playhead,
        )
    }

    pub fn seek_crossfade_in(&self) -> SmoothOutput<f32> {
        self.seek_crossfade_in.output()
    }

    pub fn seek_crossfade_out(&self) -> (SmoothOutput<f32>, SampleTime) {
        (
            self.seek_crossfade_out.output(),
            self.seek_crossfade_out_playhead,
        )
    }
}
