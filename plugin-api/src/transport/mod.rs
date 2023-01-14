use clack_host::events::event_types::TransportEvent;

mod declick;

pub use declick::{DeclickBuffers, DeclickInfo};

#[derive(Clone)]
pub struct TransportInfo {
    playhead_frame: u64,
    is_playing: bool,
    loop_state: LoopState,
    loop_back_info: Option<LoopBackInfo>,
    seek_info: Option<SeekInfo>,
    range_checker: RangeChecker,
    event: Option<TransportEvent>,
    declick: DeclickInfo,
}

impl std::fmt::Debug for TransportInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("TransportInfo");

        f.field("playhead_frame", &self.playhead_frame);
        f.field("is_playing", &self.is_playing);
        f.field("loop_state", &self.loop_state);
        f.field("loop_back_info", &self.loop_back_info);
        f.field("seek_info", &self.seek_info);
        f.field("range_checker", &self.range_checker);

        f.finish()
    }
}

impl TransportInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn _new(
        playhead_frame: u64,
        is_playing: bool,
        loop_state: LoopState,
        loop_back_info: Option<LoopBackInfo>,
        seek_info: Option<SeekInfo>,
        range_checker: RangeChecker,
        event: Option<TransportEvent>,
        declick: DeclickInfo,
    ) -> Self {
        Self {
            playhead_frame,
            is_playing,
            loop_state,
            loop_back_info,
            seek_info,
            range_checker,
            event,
            declick,
        }
    }

    /// When `plackback_state()` is of type `Playing`, then this position is the frame at the start
    /// of this process block. (And `playhead + proc_info.frames` is the end position (exclusive) of
    /// this process block.)
    pub fn playhead_frame(&self) -> u64 {
        self.playhead_frame
    }

    /// Whether or not the timeline is playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// The state of looping on the timeline transport.
    pub fn loop_state(&self) -> LoopState {
        self.loop_state
    }

    /// Returns `Some` if the transport is looping back on this current process cycle.
    pub fn do_loop_back(&self) -> Option<&LoopBackInfo> {
        self.loop_back_info.as_ref()
    }

    /// Returns `Some` if the transport has seeked to a new position this current process cycle.
    pub fn did_seek(&self) -> Option<&SeekInfo> {
        self.seek_info.as_ref()
    }

    /// Use this to check whether a range of frames lies inside this current process block.
    ///
    /// This will properly handle playing, paused, and looping conditions.
    ///
    /// This will always return false when the transport status is `Paused` or `Clear`.
    ///
    /// * `start` - The start of the range (inclusive).
    /// * `end` - The end of the range (exclusive).
    pub fn is_range_active(&self, start: u64, end: u64) -> bool {
        self.range_checker.is_range_active(self.playhead_frame, start, end)
    }

    /// Use this to check whether a particular frame lies inside this current process block.
    ///
    /// This will properly handle playing, paused, and looping conditions.
    ///
    /// This will always return false when the transport status is `Paused` or `Clear`.
    pub fn is_frame_active(&self, frame: u64) -> bool {
        self.range_checker.is_frame_active(self.playhead_frame, frame)
    }

    pub fn event(&self) -> Option<&TransportEvent> {
        self.event.as_ref()
    }

    pub fn declick_info(&self) -> &DeclickInfo {
        &self.declick
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoopBackInfo {
    /// The frame where the loop starts on the timeline (inclusive).
    pub loop_start: u64,

    /// The frame where the loop ends on the timeline (exclusive).
    pub loop_end: u64,

    /// The frame where the playhead will end on this current process cycle (exclusive).
    pub playhead_end: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct SeekInfo {
    /// This is what the playhead would have been if the transport did not seek this
    /// process cycle.
    pub seeked_from_playhead: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum RangeChecker {
    Playing {
        /// The end frame (exclusive)
        end_frame: u64,
    },
    Looping {
        /// The end frame of the first part before the loop-back (exclusive)
        end_frame_1: u64,
        /// The start frame of the second part after the loop-back (inclusive)
        start_frame_2: u64,
        /// The end frame of the second part after the loop-back (exclusive)
        end_frame_2: u64,
    },
    Paused,
}

impl RangeChecker {
    #[inline]
    pub fn is_range_active(&self, playhead: u64, start: u64, end: u64) -> bool {
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
    pub fn is_frame_active(&self, playhead: u64, frame: u64) -> bool {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopState {
    /// The transport is not currently looping.
    Inactive,
    /// The transport is currently looping.
    Active {
        /// The start of the loop (inclusive).
        loop_start_frame: u64,
        /// The end of the loop (exclusive).
        loop_end_frame: u64,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn transport_range_checker() {
        use super::RangeChecker;

        let playhead = 3;
        let r = RangeChecker::Playing { end_frame: 10 };

        assert!(r.is_range_active(playhead, 5, 12));
        assert!(r.is_range_active(playhead, 0, 5));
        assert!(r.is_range_active(playhead, 3, 10));
        assert!(!r.is_range_active(playhead, 10, 12));
        assert!(!r.is_range_active(playhead, 12, 14));
        assert!(r.is_range_active(playhead, 9, 12));
        assert!(!r.is_range_active(playhead, 0, 2));
        assert!(!r.is_range_active(playhead, 0, 3));
        assert!(r.is_range_active(playhead, 0, 4));
        assert!(r.is_range_active(playhead, 4, 8));

        assert!(!r.is_frame_active(playhead, 0));
        assert!(!r.is_frame_active(playhead, 2));
        assert!(r.is_frame_active(playhead, 3));
        assert!(r.is_frame_active(playhead, 9));
        assert!(!r.is_frame_active(playhead, 10));
        assert!(!r.is_frame_active(playhead, 11));

        let playhead = 20;
        let r = RangeChecker::Looping { end_frame_1: 24, start_frame_2: 2, end_frame_2: 10 };

        assert!(r.is_range_active(playhead, 0, 5));
        assert!(r.is_range_active(playhead, 0, 3));
        assert!(!r.is_range_active(playhead, 0, 2));
        assert!(r.is_range_active(playhead, 15, 27));
        assert!(r.is_range_active(playhead, 15, 21));
        assert!(!r.is_range_active(playhead, 15, 20));
        assert!(r.is_range_active(playhead, 4, 23));
        assert!(r.is_range_active(playhead, 0, 30));
        assert!(!r.is_range_active(playhead, 10, 18));
        assert!(!r.is_range_active(playhead, 12, 20));

        assert!(!r.is_frame_active(playhead, 0));
        assert!(r.is_frame_active(playhead, 2));
        assert!(r.is_frame_active(playhead, 3));
        assert!(!r.is_frame_active(playhead, 10));
        assert!(!r.is_frame_active(playhead, 15));
        assert!(r.is_frame_active(playhead, 20));
        assert!(r.is_frame_active(playhead, 23));
        assert!(!r.is_frame_active(playhead, 24));
        assert!(!r.is_frame_active(playhead, 25));
    }
}
