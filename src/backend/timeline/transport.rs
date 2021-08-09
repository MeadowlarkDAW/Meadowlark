use std::fmt::Debug;

use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_time::{MusicalTime, SampleRate, SampleTime, Seconds, TempoMap};

use super::AudioClipDeclick;
use crate::backend::{graph_interface::ProcInfo, MAX_BLOCKSIZE};

#[derive(Debug, Clone, Copy)]
pub struct TimelineTransportSaveState {
    pub seek_to: MusicalTime,
    pub loop_state: LoopState,
}

impl Default for TimelineTransportSaveState {
    fn default() -> Self {
        Self {
            seek_to: MusicalTime::new(0.0),
            loop_state: LoopState::Inactive,
        }
    }
}

pub struct TimelineTransportHandle {
    parameters: Shared<SharedCell<Parameters>>,
    coll_handle: Handle,

    seek_to_version: u64,
}

impl TimelineTransportHandle {
    pub fn seek_to(
        &mut self,
        seek_to: MusicalTime,
        save_state: &mut TimelineTransportSaveState,
        tempo_map: &TempoMap,
    ) {
        save_state.seek_to = seek_to;

        self.seek_to_version += 1;
        let mut params = Parameters::clone(&self.parameters.get());
        params.seek_to = (
            seek_to.to_nearest_sample_round(tempo_map),
            self.seek_to_version,
        );
        self.parameters.set(Shared::new(&self.coll_handle, params));
    }

    pub fn set_playing(&mut self, playing: bool) {
        let mut params = Parameters::clone(&self.parameters.get());
        params.is_playing = playing;
        self.parameters.set(Shared::new(&self.coll_handle, params));
    }

    /// Set the looping state.
    ///
    /// This will return an error if `loop_end - loop_start` is less than `MAX_BLOCKSIZE` (128).
    pub fn set_loop_state(
        &mut self,
        loop_state: LoopState,
        save_state: &mut TimelineTransportSaveState,
        tempo_map: &TempoMap,
    ) -> Result<(), ()> {
        if let LoopState::Active {
            loop_start,
            loop_end,
        } = loop_state
        {
            let loop_start_smp = loop_start.to_nearest_sample_round(tempo_map);
            let loop_end_smp = loop_end.to_nearest_sample_round(tempo_map);

            // Make sure loop is valid.
            if loop_end_smp - loop_start_smp < SampleTime::new(MAX_BLOCKSIZE as i64) {
                return Err(());
            }
        }

        save_state.loop_state = loop_state;

        let mut params = Parameters::clone(&self.parameters.get());
        params.loop_state = loop_state.to_proc_info(tempo_map);
        self.parameters.set(Shared::new(&self.coll_handle, params));

        Ok(())
    }

    pub fn update_tempo_map(
        &mut self,
        tempo_map: &TempoMap,
        save_state: &TimelineTransportSaveState,
    ) {
        self.seek_to_version += 1;
        let mut params = Parameters::clone(&self.parameters.get());
        params.seek_to = (
            save_state.seek_to.to_nearest_sample_round(tempo_map),
            self.seek_to_version,
        );
        params.loop_state = save_state.loop_state.to_proc_info(tempo_map);
        self.parameters.set(Shared::new(&self.coll_handle, params));
    }
}

#[derive(Debug, Clone, Copy)]
struct Parameters {
    seek_to: (SampleTime, u64),
    is_playing: bool,
    loop_state: LoopStateProcInfo,
}

/// The state of the timeline transport.
pub struct TimelineTransport {
    parameters: Shared<SharedCell<Parameters>>,

    playhead: SampleTime,
    is_playing: bool,

    loop_state: LoopStateProcInfo,

    loop_back_info: Option<LoopBackInfo>,
    seek_info: Option<SeekInfo>,

    range_checker: RangeChecker,
    next_playhead: SampleTime,

    audio_clip_declick: Option<AudioClipDeclick>,

    seek_to_version: u64,
}

impl Debug for TimelineTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "playhead: {:?}, is_playing: {:?}, loop_state: {:?}, loop_back_info: {:?}, seek_info: {:?}, range_checker {:?}, next_playhead: {:?}, seek_to_version: {:?}",
            self.playhead,
            self.is_playing,
            self.loop_state,
            self.loop_back_info,
            self.seek_info,
            self.range_checker,
            self.next_playhead,
            self.seek_to_version
        )
    }
}

impl TimelineTransport {
    pub fn new(
        save_state: &TimelineTransportSaveState,
        coll_handle: Handle,
        sample_rate: SampleRate,
        declick_time: Seconds,
        tempo_map: &TempoMap,
    ) -> (Self, TimelineTransportHandle) {
        // Make sure we are given a valid save state.
        if let LoopState::Active {
            loop_start,
            loop_end,
        } = save_state.loop_state
        {
            let loop_start_smp = loop_start.to_nearest_sample_round(tempo_map);
            let loop_end_smp = loop_end.to_nearest_sample_round(tempo_map);

            // Make sure loop is valid.
            assert!(loop_end_smp - loop_start_smp >= SampleTime::new(MAX_BLOCKSIZE as i64));
        }

        let parameters = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                Parameters {
                    seek_to: (save_state.seek_to.to_nearest_sample_round(tempo_map), 0),
                    is_playing: false,
                    loop_state: save_state.loop_state.to_proc_info(tempo_map),
                },
            )),
        );

        let playhead = save_state.seek_to.to_nearest_sample_round(tempo_map);

        (
            TimelineTransport {
                parameters: Shared::clone(&parameters),
                playhead,
                is_playing: false,
                loop_state: save_state.loop_state.to_proc_info(tempo_map),
                loop_back_info: None,
                seek_info: None,
                range_checker: RangeChecker::Paused,
                next_playhead: playhead,
                audio_clip_declick: Some(AudioClipDeclick::new(declick_time, sample_rate)),
                seek_to_version: 0,
            },
            TimelineTransportHandle {
                parameters,
                coll_handle,
                seek_to_version: 0,
            },
        )
    }

    /// Update the state of this transport.
    pub fn update(&mut self, frames: usize) {
        let Parameters {
            seek_to,
            is_playing,
            loop_state,
        } = *self.parameters.get();

        let frames = SampleTime::from_usize(frames);

        // Seek if gotten a new version of the seek_to value.
        self.seek_info = None;
        if self.seek_to_version != seek_to.1 {
            self.seek_to_version = seek_to.1;

            self.seek_info = Some(SeekInfo {
                seeked_from_playhead: self.next_playhead,
            });

            self.playhead = seek_to.0;
            self.next_playhead = seek_to.0;
        };

        self.is_playing = is_playing;
        self.loop_state = loop_state;
        self.loop_back_info = None;
        if self.is_playing {
            self.playhead = self.next_playhead;

            // Advance the playhead.
            let mut did_loop = false;
            if let LoopStateProcInfo::Active {
                loop_start,
                loop_end,
            } = loop_state
            {
                if self.playhead < loop_end && self.playhead + frames > loop_end {
                    let first_frames = loop_end - self.playhead;
                    let second_frames = frames - first_frames;

                    self.range_checker = RangeChecker::Looping {
                        end_frame_1: loop_end,
                        start_frame_2: loop_start,
                        end_frame_2: loop_start + second_frames,
                    };

                    self.next_playhead = loop_start + second_frames;

                    self.loop_back_info = Some(LoopBackInfo {
                        loop_start,
                        loop_end,
                        playhead_end: self.next_playhead,
                    });

                    did_loop = true;
                }
            }

            if !did_loop {
                self.next_playhead = self.playhead + frames;

                self.range_checker = RangeChecker::Playing {
                    end_frame: self.next_playhead,
                };
            }
        } else {
            self.range_checker = RangeChecker::Paused;
        }
    }

    pub fn process_declicker(&mut self, proc_info: &ProcInfo) {
        // Get around borrow checker.
        let mut audio_clip_declick = self.audio_clip_declick.take().unwrap();
        audio_clip_declick.process(proc_info, self);
        self.audio_clip_declick = Some(audio_clip_declick);
    }

    /// When `plackback_state()` is of type `Playing`, then this position is the frame at the start
    /// of this process block. (And `playhead + proc_info.frames` is the end position (exclusive) of
    /// this process block.)
    #[inline]
    pub fn playhead(&self) -> SampleTime {
        self.playhead
    }

    /// Whether or not the timeline is playing.
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// The state of looping on the timeline transport.
    #[inline]
    pub fn loop_state(&self) -> LoopStateProcInfo {
        self.loop_state
    }

    /// Returns `Some` if the transport is looping back on this current process cycle.
    #[inline]
    pub fn do_loop_back(&self) -> Option<&LoopBackInfo> {
        self.loop_back_info.as_ref()
    }

    /// Returns `Some` if the transport has seeked to a new position this current process cycle.
    #[inline]
    pub fn did_seek(&self) -> Option<&SeekInfo> {
        self.seek_info.as_ref()
    }

    /// Use this to check whether a range of samples lies inside this current process block.
    ///
    /// This will properly handle playing, paused, and looping conditions.
    ///
    /// This will always return false when the transport status is `Paused` or `Clear`.
    ///
    /// * `start` - The start of the range (inclusive).
    /// * `end` - The end of the range (exclusive).
    pub fn is_range_active(&self, start: SampleTime, end: SampleTime) -> bool {
        self.range_checker
            .is_range_active(self.playhead, start, end)
    }

    /// Use this to check whether a particular sample lies inside this current process block.
    ///
    /// This will properly handle playing, paused, and looping conditions.
    ///
    /// This will always return false when the transport status is `Paused` or `Clear`.
    pub fn is_sample_active(&self, sample: SampleTime) -> bool {
        self.range_checker.is_sample_active(self.playhead, sample)
    }

    /// Returns the audio clip declicker helper struct.
    pub fn audio_clip_declick(&self) -> &AudioClipDeclick {
        self.audio_clip_declick.as_ref().unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoopBackInfo {
    /// The frame where the loop starts on the timeline (inclusive).
    pub loop_start: SampleTime,

    /// The frame where the loop ends on the timeline (exclusive).
    pub loop_end: SampleTime,

    /// The frame where the playhead will end on this current process cycle (exclusive).
    pub playhead_end: SampleTime,
}

#[derive(Debug, Clone, Copy)]
pub struct SeekInfo {
    /// This is what the playhead would have been if the transport did not seek this
    /// process cycle.
    pub seeked_from_playhead: SampleTime,
}

#[derive(Debug, Clone, Copy)]
enum RangeChecker {
    Playing {
        end_frame: SampleTime,
    },
    Looping {
        end_frame_1: SampleTime,
        start_frame_2: SampleTime,
        end_frame_2: SampleTime,
    },
    Paused,
}

impl RangeChecker {
    #[inline]
    pub fn is_range_active(
        &self,
        playhead: SampleTime,
        start: SampleTime,
        end: SampleTime,
    ) -> bool {
        match self {
            RangeChecker::Playing { end_frame } => playhead < end && start < *end_frame,
            RangeChecker::Looping {
                end_frame_1,
                start_frame_2,
                end_frame_2,
            } => {
                (playhead < end && start < *end_frame_1)
                    || (*start_frame_2 < end && start < *end_frame_2)
            }
            RangeChecker::Paused => false,
        }
    }
    #[inline]
    pub fn is_sample_active(&self, playhead: SampleTime, sample: SampleTime) -> bool {
        match self {
            RangeChecker::Playing { end_frame } => sample >= playhead && sample < *end_frame,
            RangeChecker::Looping {
                end_frame_1,
                start_frame_2,
                end_frame_2,
            } => {
                (sample >= playhead && sample < *end_frame_1)
                    || (sample >= *start_frame_2 && sample < *end_frame_2)
            }
            RangeChecker::Paused => false,
        }
    }
}

/// The status of looping on this transport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopState {
    /// The transport is not currently looping.
    Inactive,
    /// The transport is currently looping.
    Active {
        /// The start of the loop (inclusive).
        loop_start: MusicalTime,
        /// The end of the loop (exclusive).
        loop_end: MusicalTime,
    },
}

impl LoopState {
    fn to_proc_info(&self, tempo_map: &TempoMap) -> LoopStateProcInfo {
        match self {
            LoopState::Inactive => LoopStateProcInfo::Inactive,
            &LoopState::Active {
                loop_start,
                loop_end,
            } => LoopStateProcInfo::Active {
                loop_start: loop_start.to_nearest_sample_round(tempo_map),
                loop_end: loop_end.to_nearest_sample_round(tempo_map),
            },
        }
    }
}

/// The status of looping on this transport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoopStateProcInfo {
    /// The transport is not currently looping.
    Inactive,
    /// The transport is currently looping.
    Active {
        /// The start of the loop (inclusive).
        loop_start: SampleTime,
        /// The end of the loop (exclusive).
        loop_end: SampleTime,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn transport_range_checker() {
        use super::RangeChecker;
        use rusty_daw_time::SampleTime;

        let playhead = SampleTime::new(3);
        let r = RangeChecker::Playing {
            end_frame: SampleTime::new(10),
        };

        // This is probably overkill, but I just needed to make sure every edge case works.

        assert!(r.is_range_active(playhead, SampleTime::new(5), SampleTime::new(12)));
        assert!(r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(5)));
        assert!(r.is_range_active(playhead, SampleTime::new(3), SampleTime::new(10)));
        assert!(!r.is_range_active(playhead, SampleTime::new(10), SampleTime::new(12)));
        assert!(!r.is_range_active(playhead, SampleTime::new(12), SampleTime::new(14)));
        assert!(r.is_range_active(playhead, SampleTime::new(9), SampleTime::new(12)));
        assert!(!r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(2)));
        assert!(!r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(3)));
        assert!(r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(4)));
        assert!(r.is_range_active(playhead, SampleTime::new(4), SampleTime::new(8)));

        assert!(!r.is_sample_active(playhead, SampleTime::new(0)));
        assert!(!r.is_sample_active(playhead, SampleTime::new(2)));
        assert!(r.is_sample_active(playhead, SampleTime::new(3)));
        assert!(r.is_sample_active(playhead, SampleTime::new(9)));
        assert!(!r.is_sample_active(playhead, SampleTime::new(10)));
        assert!(!r.is_sample_active(playhead, SampleTime::new(11)));

        let playhead = SampleTime::new(20);
        let r = RangeChecker::Looping {
            end_frame_1: SampleTime::new(24),
            start_frame_2: SampleTime::new(2),
            end_frame_2: SampleTime::new(10),
        };

        assert!(r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(5)));
        assert!(r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(3)));
        assert!(!r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(2)));
        assert!(r.is_range_active(playhead, SampleTime::new(15), SampleTime::new(27)));
        assert!(r.is_range_active(playhead, SampleTime::new(15), SampleTime::new(21)));
        assert!(!r.is_range_active(playhead, SampleTime::new(15), SampleTime::new(20)));
        assert!(r.is_range_active(playhead, SampleTime::new(4), SampleTime::new(23)));
        assert!(r.is_range_active(playhead, SampleTime::new(0), SampleTime::new(30)));
        assert!(!r.is_range_active(playhead, SampleTime::new(10), SampleTime::new(18)));
        assert!(!r.is_range_active(playhead, SampleTime::new(12), SampleTime::new(20)));

        assert!(!r.is_sample_active(playhead, SampleTime::new(0)));
        assert!(r.is_sample_active(playhead, SampleTime::new(2)));
        assert!(r.is_sample_active(playhead, SampleTime::new(3)));
        assert!(!r.is_sample_active(playhead, SampleTime::new(10)));
        assert!(!r.is_sample_active(playhead, SampleTime::new(15)));
        assert!(r.is_sample_active(playhead, SampleTime::new(20)));
        assert!(r.is_sample_active(playhead, SampleTime::new(23)));
        assert!(!r.is_sample_active(playhead, SampleTime::new(24)));
        assert!(!r.is_sample_active(playhead, SampleTime::new(25)));
    }
}
