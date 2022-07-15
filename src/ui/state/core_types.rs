use meadowlark_core_types::{Frames, MusicalTime, SampleRate, Seconds, SuperFrames};
use std::hash::Hash;
use vizia::prelude::Data;

/// A wrapper around `meadowlark_core_types::SampleRate` so we can derive
/// `vizia::Data` on it.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Data)]
pub struct WSampleRate(f64);

impl WSampleRate {
    pub const fn new(s: SampleRate) -> Self {
        Self(s.0)
    }

    pub fn get(&self) -> SampleRate {
        SampleRate(self.0)
    }
}

impl From<SampleRate> for WSampleRate {
    fn from(s: SampleRate) -> Self {
        Self::new(s)
    }
}

impl From<WSampleRate> for SampleRate {
    fn from(s: WSampleRate) -> Self {
        s.get()
    }
}

/// A wrapper around `meadowlark_core_types::MusicalTime` so we can derive
/// `vizia::Data` on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Data)]
pub struct WMusicalTime {
    beats: u32,
    super_beats: u32,
}

impl WMusicalTime {
    pub fn new(m: MusicalTime) -> Self {
        Self { beats: m.beats(), super_beats: m.super_beats() }
    }

    pub fn get(&self) -> MusicalTime {
        MusicalTime::new(self.beats, self.super_beats)
    }
}

impl From<MusicalTime> for WMusicalTime {
    fn from(m: MusicalTime) -> Self {
        Self::new(m)
    }
}

impl From<WMusicalTime> for MusicalTime {
    fn from(m: WMusicalTime) -> Self {
        m.get()
    }
}

/// A wrapper around `meadowlark_core_types::Seconds` so we can derive
/// `vizia::Data` on it.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Data)]
pub struct WSeconds(f64);

impl WSeconds {
    pub const fn new(s: Seconds) -> Self {
        Self(s.0)
    }

    pub fn get(&self) -> Seconds {
        Seconds(self.0)
    }
}

impl From<Seconds> for WSeconds {
    fn from(s: Seconds) -> Self {
        Self::new(s)
    }
}

impl From<WSeconds> for Seconds {
    fn from(s: WSeconds) -> Self {
        s.get()
    }
}

/// A wrapper around `meadowlark_core_types::Frames` so we can derive
/// `vizia::Data` on it.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, Data)]
pub struct WFrames(u64);

impl WFrames {
    pub const fn new(s: Frames) -> Self {
        Self(s.0)
    }

    pub fn get(&self) -> Frames {
        Frames(self.0)
    }
}

impl From<Frames> for WFrames {
    fn from(s: Frames) -> Self {
        Self::new(s)
    }
}

impl From<WFrames> for Frames {
    fn from(s: WFrames) -> Self {
        s.get()
    }
}

/// A wrapper around `meadowlark_core_types::SuperFrames` so we can derive
/// `vizia::Data` on it.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, Data)]
pub struct WSuperFrames(u64);

impl WSuperFrames {
    pub const fn new(s: SuperFrames) -> Self {
        Self(s.0)
    }

    pub fn get(&self) -> SuperFrames {
        SuperFrames(self.0)
    }
}

impl From<SuperFrames> for WSuperFrames {
    fn from(s: SuperFrames) -> Self {
        Self::new(s)
    }
}

impl From<WSuperFrames> for SuperFrames {
    fn from(s: WSuperFrames) -> Self {
        s.get()
    }
}
