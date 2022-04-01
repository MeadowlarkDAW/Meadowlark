use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use basedrop::{Handle, Shared, SharedCell};
use rusty_daw_core::{Frames, MusicalTime, ProcFrames, SampleRate};

use super::audio_clip::AudioClipDeclick;
use super::{TempoMap, TimelineTransportState};

pub struct TimelineTransportHandle<const MAX_BLOCKSIZE: usize> {
    parameters: Shared<SharedCell<Parameters>>,

    tempo_map_shared: Shared<SharedCell<(Shared<TempoMap>, u64)>>,
    tempo_map: Shared<TempoMap>,

    playhead_shared: Arc<AtomicU64>,
    playhead_frames: Frames,
    playhead: MusicalTime,

    tempo_map_version: u64,

    coll_handle: Handle,
}

impl<const MAX_BLOCKSIZE: usize> TimelineTransportHandle<MAX_BLOCKSIZE> {
    pub fn seek_to(&mut self, seek_to: MusicalTime, state: &mut TimelineTransportState) {
        state.seek_to = seek_to;

        let mut params = Parameters::clone(&self.parameters.get());
        params.seek_to = (seek_to, params.seek_to.1 + 1);
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
        state: &mut TimelineTransportState,
    ) -> Result<(), ()> {
        if let LoopState::Active { loop_start, loop_end } = loop_state {
            let loop_start_frame = self.tempo_map.musical_to_nearest_frame_round(loop_start);
            let loop_end_frame = self.tempo_map.musical_to_nearest_frame_round(loop_end);

            if loop_start_frame >= loop_end_frame {
                return Err(());
            }

            // The looping algorithm will only work if the loop range is >= MAX_BLOCKSIZE.
            if loop_end_frame - loop_start_frame < Frames::new(MAX_BLOCKSIZE as u64) {
                return Err(());
            }
        }

        state.loop_state = loop_state;

        let mut params = Parameters::clone(&self.parameters.get());
        params.loop_state = (loop_state, params.loop_state.1 + 1);
        self.parameters.set(Shared::new(&self.coll_handle, params));

        Ok(())
    }

    pub fn get_playhead_position(&mut self) -> MusicalTime {
        let new_pos_frames = Frames::new(self.playhead_shared.load(Ordering::Relaxed));
        if self.playhead_frames != new_pos_frames {
            self.playhead_frames = new_pos_frames;
            self.playhead = self.tempo_map.frame_to_musical(new_pos_frames);
        }

        self.playhead
    }

    /// Only to be used by the `ProjectStateInterface` struct. If used anywhere else, it could cause
    /// shared state to become desynchronized.
    pub fn _update_tempo_map(&mut self, tempo_map: TempoMap) {
        let tempo_map = Shared::new(&self.coll_handle, tempo_map);

        self.tempo_map_version += 1;
        self.tempo_map_shared.set(Shared::new(
            &self.coll_handle,
            (Shared::clone(&tempo_map), self.tempo_map_version),
        ));
        self.tempo_map = tempo_map;
    }
}

#[derive(Debug, Clone, Copy)]
struct Parameters {
    seek_to: (MusicalTime, u64),
    is_playing: bool,
    loop_state: (LoopState, u64),
}

/// The state of the timeline transport.
pub struct TimelineTransport<const MAX_BLOCKSIZE: usize> {
    parameters: Shared<SharedCell<Parameters>>,

    tempo_map_shared: Shared<SharedCell<(Shared<TempoMap>, u64)>>,
    tempo_map: Shared<TempoMap>,

    playhead: Frames,
    is_playing: bool,

    loop_state: LoopStateProcInfo,

    loop_back_info: Option<LoopBackInfo>,
    seek_info: Option<SeekInfo>,

    range_checker: RangeChecker,
    next_playhead: Frames,

    audio_clip_declick: Option<AudioClipDeclick<MAX_BLOCKSIZE>>,

    seek_to_version: u64,
    loop_state_version: u64,
    tempo_map_version: u64,

    tempo_map_changed: bool,

    playhead_shared: Arc<AtomicU64>,
}

impl<const MAX_BLOCKSIZE: usize> Debug for TimelineTransport<MAX_BLOCKSIZE> {
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

impl<const MAX_BLOCKSIZE: usize> TimelineTransport<MAX_BLOCKSIZE> {
    pub fn new(
        coll_handle: Handle,
        sample_rate: SampleRate,
    ) -> (Self, TimelineTransportHandle<MAX_BLOCKSIZE>) {
        let state = TimelineTransportState::default();

        let parameters = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                Parameters {
                    seek_to: (state.seek_to, 0),
                    is_playing: false,
                    loop_state: (state.loop_state, 0),
                },
            )),
        );

        let tempo_map = TempoMap::new(120.0, sample_rate);

        let playhead = tempo_map.musical_to_nearest_frame_round(state.seek_to);
        let playhead_shared = Arc::new(AtomicU64::new(playhead.0));
        let loop_state = state.loop_state.to_proc_info(&tempo_map);

        let tempo_map = Shared::new(&coll_handle, tempo_map);
        let tempo_map_shared = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(&coll_handle, (Shared::clone(&tempo_map), 0))),
        );

        (
            TimelineTransport {
                parameters: Shared::clone(&parameters),
                tempo_map_shared: Shared::clone(&tempo_map_shared),
                tempo_map: Shared::clone(&tempo_map),
                playhead,
                is_playing: false,
                loop_state,
                loop_back_info: None,
                seek_info: None,
                range_checker: RangeChecker::Paused,
                next_playhead: playhead,
                audio_clip_declick: Some(AudioClipDeclick::new(sample_rate)),
                seek_to_version: 0,
                tempo_map_version: 0,
                loop_state_version: 0,
                tempo_map_changed: false,
                playhead_shared: Arc::clone(&playhead_shared),
            },
            TimelineTransportHandle {
                parameters,
                tempo_map_shared,
                tempo_map,
                coll_handle,
                tempo_map_version: 0,
                playhead_shared,
                playhead_frames: playhead,
                playhead: state.seek_to,
            },
        )
    }

    /// Update the state of this transport.
    pub fn process(&mut self, proc_frames: ProcFrames<MAX_BLOCKSIZE>) {
        let Parameters { seek_to, is_playing, loop_state } = *self.parameters.get();

        self.playhead = self.next_playhead;

        let mut loop_state_changed = false;
        if self.loop_state_version != loop_state.1 {
            self.loop_state_version = loop_state.1;
            loop_state_changed = true;
        }

        // Check if the tempo map has changed.
        self.tempo_map_changed = false;
        let (new_tempo_map, new_version) = &*self.tempo_map_shared.get();
        if self.tempo_map_version != *new_version {
            self.tempo_map_version = *new_version;

            // Get musical time of the playhead using the old tempo map.
            let playhead = self.tempo_map.frame_to_musical(self.playhead);

            // Make sure the audio clip declicker updates it internal playheads.
            self.audio_clip_declick
                .as_mut()
                .unwrap()
                .update_tempo_map(&*self.tempo_map, &*new_tempo_map);

            self.tempo_map = Shared::clone(new_tempo_map);
            self.tempo_map_changed = true;

            // Update proc info.
            self.playhead = self.tempo_map.musical_to_nearest_frame_round(playhead);
            loop_state_changed = true;
        }

        // Seek if gotten a new version of the seek_to value.
        self.seek_info = None;
        if self.seek_to_version != seek_to.1 {
            self.seek_to_version = seek_to.1;

            self.seek_info = Some(SeekInfo { seeked_from_playhead: self.playhead });

            self.playhead = self.tempo_map.musical_to_nearest_frame_round(seek_to.0);
            self.next_playhead = self.playhead;
        };

        if loop_state_changed {
            self.loop_state = match loop_state.0 {
                LoopState::Inactive => LoopStateProcInfo::Inactive,
                LoopState::Active { loop_start, loop_end } => LoopStateProcInfo::Active {
                    loop_start: self.tempo_map.musical_to_nearest_frame_round(loop_start),
                    loop_end: self.tempo_map.musical_to_nearest_frame_round(loop_end),
                },
            };
        }

        self.is_playing = is_playing;
        self.loop_back_info = None;
        self.playhead = self.next_playhead;
        if self.is_playing {
            // Advance the playhead.
            let mut did_loop = false;
            if let LoopStateProcInfo::Active { loop_start, loop_end } = self.loop_state {
                if self.playhead < loop_end
                    && self.playhead + Frames::from_proc_frames(proc_frames) >= loop_end
                {
                    let first_frames = loop_end - self.playhead;
                    let second_frames = Frames::from_proc_frames(proc_frames) - first_frames;

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
                self.next_playhead = self.playhead + Frames::from_proc_frames(proc_frames);

                self.range_checker = RangeChecker::Playing { end_frame: self.next_playhead };
            }
        } else {
            self.range_checker = RangeChecker::Paused;
        }

        self.playhead_shared.store(self.playhead.0, Ordering::Relaxed);

        // Get around borrow checker.
        let mut audio_clip_declick = self.audio_clip_declick.take().unwrap();
        audio_clip_declick.process(proc_frames, self);
        self.audio_clip_declick = Some(audio_clip_declick);
    }

    /// When `plackback_state()` is of type `Playing`, then this position is the frame at the start
    /// of this process block. (And `playhead + proc_info.frames` is the end position (exclusive) of
    /// this process block.)
    #[inline]
    pub fn playhead(&self) -> Frames {
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

    /// Returns true if the tempo map has changed this current process cycle.
    #[inline]
    pub fn did_tempo_map_change(&self) -> bool {
        self.tempo_map_changed
    }

    /// Returns the current tempo map.
    #[inline]
    pub fn tempo_map(&self) -> &TempoMap {
        &*self.tempo_map
    }

    /// Use this to check whether a range of frames lies inside this current process block.
    ///
    /// This will properly handle playing, paused, and looping conditions.
    ///
    /// This will always return false when the transport status is `Paused` or `Clear`.
    ///
    /// * `start` - The start of the range (inclusive).
    /// * `end` - The end of the range (exclusive).
    pub fn is_range_active(&self, start: Frames, end: Frames) -> bool {
        self.range_checker.is_range_active(self.playhead, start, end)
    }

    /// Use this to check whether a particular frame lies inside this current process block.
    ///
    /// This will properly handle playing, paused, and looping conditions.
    ///
    /// This will always return false when the transport status is `Paused` or `Clear`.
    pub fn is_frame_active(&self, frame: Frames) -> bool {
        self.range_checker.is_frame_active(self.playhead, frame)
    }

    /// Returns the audio clip declicker helper struct.
    pub fn audio_clip_declick(&self) -> &AudioClipDeclick<MAX_BLOCKSIZE> {
        self.audio_clip_declick.as_ref().unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoopBackInfo {
    /// The frame where the loop starts on the timeline (inclusive).
    pub loop_start: Frames,

    /// The frame where the loop ends on the timeline (exclusive).
    pub loop_end: Frames,

    /// The frame where the playhead will end on this current process cycle (exclusive).
    pub playhead_end: Frames,
}

#[derive(Debug, Clone, Copy)]
pub struct SeekInfo {
    /// This is what the playhead would have been if the transport did not seek this
    /// process cycle.
    pub seeked_from_playhead: Frames,
}

#[derive(Debug, Clone, Copy)]
enum RangeChecker {
    Playing { end_frame: Frames },
    Looping { end_frame_1: Frames, start_frame_2: Frames, end_frame_2: Frames },
    Paused,
}

impl RangeChecker {
    #[inline]
    pub fn is_range_active(&self, playhead: Frames, start: Frames, end: Frames) -> bool {
        match self {
            RangeChecker::Playing { end_frame } => playhead < end && start < *end_frame,
            RangeChecker::Looping { end_frame_1, start_frame_2, end_frame_2 } => {
                (playhead < end && start < *end_frame_1)
                    || (*start_frame_2 < end && start < *end_frame_2)
            }
            RangeChecker::Paused => false,
        }
    }
    #[inline]
    pub fn is_frame_active(&self, playhead: Frames, frame: Frames) -> bool {
        match self {
            RangeChecker::Playing { end_frame } => frame >= playhead && frame < *end_frame,
            RangeChecker::Looping { end_frame_1, start_frame_2, end_frame_2 } => {
                (frame >= playhead && frame < *end_frame_1)
                    || (frame >= *start_frame_2 && frame < *end_frame_2)
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
            &LoopState::Active { loop_start, loop_end } => LoopStateProcInfo::Active {
                loop_start: tempo_map.musical_to_nearest_frame_round(loop_start),
                loop_end: tempo_map.musical_to_nearest_frame_round(loop_end),
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
        loop_start: Frames,
        /// The end of the loop (exclusive).
        loop_end: Frames,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn transport_range_checker() {
        use super::RangeChecker;
        use rusty_daw_core::Frames;

        let playhead = Frames::new(3);
        let r = RangeChecker::Playing { end_frame: Frames::new(10) };

        // This is probably overkill, but I just needed to make sure every edge case works.

        assert!(r.is_range_active(playhead, Frames::new(5), Frames::new(12)));
        assert!(r.is_range_active(playhead, Frames::new(0), Frames::new(5)));
        assert!(r.is_range_active(playhead, Frames::new(3), Frames::new(10)));
        assert!(!r.is_range_active(playhead, Frames::new(10), Frames::new(12)));
        assert!(!r.is_range_active(playhead, Frames::new(12), Frames::new(14)));
        assert!(r.is_range_active(playhead, Frames::new(9), Frames::new(12)));
        assert!(!r.is_range_active(playhead, Frames::new(0), Frames::new(2)));
        assert!(!r.is_range_active(playhead, Frames::new(0), Frames::new(3)));
        assert!(r.is_range_active(playhead, Frames::new(0), Frames::new(4)));
        assert!(r.is_range_active(playhead, Frames::new(4), Frames::new(8)));

        assert!(!r.is_frame_active(playhead, Frames::new(0)));
        assert!(!r.is_frame_active(playhead, Frames::new(2)));
        assert!(r.is_frame_active(playhead, Frames::new(3)));
        assert!(r.is_frame_active(playhead, Frames::new(9)));
        assert!(!r.is_frame_active(playhead, Frames::new(10)));
        assert!(!r.is_frame_active(playhead, Frames::new(11)));

        let playhead = Frames::new(20);
        let r = RangeChecker::Looping {
            end_frame_1: Frames::new(24),
            start_frame_2: Frames::new(2),
            end_frame_2: Frames::new(10),
        };

        assert!(r.is_range_active(playhead, Frames::new(0), Frames::new(5)));
        assert!(r.is_range_active(playhead, Frames::new(0), Frames::new(3)));
        assert!(!r.is_range_active(playhead, Frames::new(0), Frames::new(2)));
        assert!(r.is_range_active(playhead, Frames::new(15), Frames::new(27)));
        assert!(r.is_range_active(playhead, Frames::new(15), Frames::new(21)));
        assert!(!r.is_range_active(playhead, Frames::new(15), Frames::new(20)));
        assert!(r.is_range_active(playhead, Frames::new(4), Frames::new(23)));
        assert!(r.is_range_active(playhead, Frames::new(0), Frames::new(30)));
        assert!(!r.is_range_active(playhead, Frames::new(10), Frames::new(18)));
        assert!(!r.is_range_active(playhead, Frames::new(12), Frames::new(20)));

        assert!(!r.is_frame_active(playhead, Frames::new(0)));
        assert!(r.is_frame_active(playhead, Frames::new(2)));
        assert!(r.is_frame_active(playhead, Frames::new(3)));
        assert!(!r.is_frame_active(playhead, Frames::new(10)));
        assert!(!r.is_frame_active(playhead, Frames::new(15)));
        assert!(r.is_frame_active(playhead, Frames::new(20)));
        assert!(r.is_frame_active(playhead, Frames::new(23)));
        assert!(!r.is_frame_active(playhead, Frames::new(24)));
        assert!(!r.is_frame_active(playhead, Frames::new(25)));
    }
}
