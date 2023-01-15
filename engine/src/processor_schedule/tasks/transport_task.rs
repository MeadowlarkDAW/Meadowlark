use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use basedrop::{Shared, SharedCell};
use clack_host::events::event_types::{TransportEvent, TransportEventFlags};
use clack_host::events::{EventFlags, EventHeader};
use meadowlark_plugin_api::transport::{
    LoopBackInfo, LoopState, RangeChecker, SeekInfo, TransportInfo,
};
use meadowlark_plugin_api::{BeatTime, SecondsTime};

use crate::engine::{EngineTempoMap, TransportInfoAtFrame};

mod declick;

use declick::{JumpInfo, TransportDeclick};

pub struct TransportHandle {
    parameters: Shared<SharedCell<Parameters>>,

    tempo_map_shared: Shared<SharedCell<(Box<dyn EngineTempoMap>, u64)>>,

    playhead_frame_shared: Arc<AtomicU64>,
    latest_playhead_frame: u64,

    last_seeked_frame: u64,

    coll_handle: basedrop::Handle,
}

impl TransportHandle {
    pub fn seek_to_frame(&mut self, seek_to_frame: u64) {
        self.last_seeked_frame = seek_to_frame;

        let mut params = Parameters::clone(&self.parameters.get());
        params.seek_to_frame = (seek_to_frame, params.seek_to_frame.1 + 1);
        self.parameters.set(Shared::new(&self.coll_handle, params));
    }

    pub fn set_playing(&mut self, playing: bool) {
        let mut params = Parameters::clone(&self.parameters.get());
        params.is_playing = playing;
        self.parameters.set(Shared::new(&self.coll_handle, params));
    }

    /// Set the looping state.
    pub fn set_loop_state(&mut self, loop_state: LoopState) {
        let mut params = Parameters::clone(&self.parameters.get());
        params.loop_state = (loop_state, params.loop_state.1 + 1);
        self.parameters.set(Shared::new(&self.coll_handle, params));
    }

    pub fn current_playhead_position_frames(&mut self) -> (u64, bool) {
        let new_playhead_frame = self.playhead_frame_shared.load(Ordering::Relaxed);
        if self.latest_playhead_frame != new_playhead_frame {
            self.latest_playhead_frame = new_playhead_frame;

            (new_playhead_frame, true)
        } else {
            (new_playhead_frame, false)
        }
    }

    pub fn current_playhead_position_beats(&mut self) -> (BeatTime, bool) {
        let new_playhead_frame = self.playhead_frame_shared.load(Ordering::Relaxed);
        let tempo_map = self.tempo_map_shared.get();

        let playhead_beats = tempo_map.0.frame_to_beat(new_playhead_frame);

        if self.latest_playhead_frame != new_playhead_frame {
            self.latest_playhead_frame = new_playhead_frame;

            (playhead_beats, true)
        } else {
            (playhead_beats, false)
        }
    }

    pub fn last_seeked_frame(&self) -> u64 {
        self.last_seeked_frame
    }

    pub fn update_tempo_map(&mut self, tempo_map: Box<dyn EngineTempoMap>) {
        let version = self.tempo_map_shared.get().1;
        SharedCell::set(
            &*self.tempo_map_shared,
            Shared::new(&self.coll_handle, (tempo_map, version + 1)),
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct Parameters {
    seek_to_frame: (u64, u64),
    is_playing: bool,
    loop_state: (LoopState, u64),
}

pub struct TransportTask {
    parameters: Shared<SharedCell<Parameters>>,

    tempo_map_shared: Shared<SharedCell<(Box<dyn EngineTempoMap>, u64)>>,

    playhead_frame: u64,
    is_playing: bool,

    loop_state: LoopState,

    loop_back_info: Option<LoopBackInfo>,
    seek_info: Option<SeekInfo>,

    range_checker: RangeChecker,
    next_playhead_frame: u64,

    seek_to_version: u64,
    loop_state_version: u64,
    tempo_map_version: u64,

    loop_start_beats: BeatTime,
    loop_end_beats: BeatTime,
    loop_start_seconds: SecondsTime,
    loop_end_seconds: SecondsTime,

    transport_info_at_frame: TransportInfoAtFrame,

    playhead_frame_shared: Arc<AtomicU64>,

    declick: TransportDeclick,
}

impl TransportTask {
    pub fn new(
        seek_to_frame: u64,
        loop_state: LoopState,
        sample_rate: u32,
        tempo_map: Box<dyn EngineTempoMap>,
        max_frames: usize,
        declick_seconds: f64,
        coll_handle: basedrop::Handle,
    ) -> (Self, TransportHandle) {
        let parameters = Shared::new(
            &coll_handle,
            SharedCell::new(Shared::new(
                &coll_handle,
                Parameters {
                    seek_to_frame: (seek_to_frame, 0),
                    is_playing: false,
                    loop_state: (loop_state, 0),
                },
            )),
        );

        let playhead_frame = seek_to_frame;
        let playhead_frame_shared = Arc::new(AtomicU64::new(playhead_frame));

        let (loop_start_beats, loop_end_beats, loop_start_seconds, loop_end_seconds) =
            if let LoopState::Active { loop_start_frame, loop_end_frame } = &loop_state {
                (
                    tempo_map.frame_to_beat(*loop_start_frame),
                    tempo_map.frame_to_beat(*loop_end_frame),
                    tempo_map.frame_to_seconds(*loop_start_frame),
                    tempo_map.frame_to_seconds(*loop_end_frame),
                )
            } else {
                (Default::default(), Default::default(), Default::default(), Default::default())
            };

        let transport_info_at_frame = tempo_map.transport_info_at_frame(playhead_frame);

        let tempo_map_shared =
            Shared::new(&coll_handle, SharedCell::new(Shared::new(&coll_handle, (tempo_map, 0))));

        let declick = TransportDeclick::new(max_frames, declick_seconds, sample_rate, &coll_handle);

        (
            TransportTask {
                parameters: Shared::clone(&parameters),
                tempo_map_shared: Shared::clone(&tempo_map_shared),
                playhead_frame,
                is_playing: false,
                loop_state,
                loop_back_info: None,
                seek_info: None,
                range_checker: RangeChecker::Paused,
                next_playhead_frame: playhead_frame,
                seek_to_version: 0,
                tempo_map_version: 0,
                loop_state_version: 0,
                loop_start_beats,
                loop_end_beats,
                loop_start_seconds,
                loop_end_seconds,
                transport_info_at_frame,
                playhead_frame_shared: Arc::clone(&playhead_frame_shared),
                declick,
            },
            TransportHandle {
                parameters,
                tempo_map_shared,
                coll_handle,
                playhead_frame_shared,
                latest_playhead_frame: playhead_frame,
                last_seeked_frame: playhead_frame,
            },
        )
    }

    /// Update the state of this transport.
    pub fn process(&mut self, frames: usize) -> TransportInfo {
        let Parameters { seek_to_frame, is_playing, loop_state } = *self.parameters.get();

        let proc_frames = frames as u64;

        self.playhead_frame = self.next_playhead_frame;

        let mut loop_state_changed = false;
        if self.loop_state_version != loop_state.1 {
            self.loop_state_version = loop_state.1;
            loop_state_changed = true;
        }

        // Check if the tempo map has changed.
        let mut tempo_map_changed = false;
        let (tempo_map, new_version) = &*self.tempo_map_shared.get();
        if self.tempo_map_version != *new_version {
            self.tempo_map_version = *new_version;

            tempo_map_changed = true;
            loop_state_changed = true;
        }

        // Seek if gotten a new version of the seek_to value.
        self.seek_info = None;
        if self.seek_to_version != seek_to_frame.1 {
            self.seek_to_version = seek_to_frame.1;

            self.seek_info = Some(SeekInfo { seeked_from_playhead: self.playhead_frame });

            self.next_playhead_frame = seek_to_frame.0
        };

        if loop_state_changed {
            let (loop_start_beats, loop_end_beats, loop_start_seconds, loop_end_seconds) =
                match &loop_state.0 {
                    LoopState::Inactive => (
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        Default::default(),
                    ),
                    LoopState::Active { loop_start_frame, loop_end_frame } => (
                        tempo_map.frame_to_beat(*loop_start_frame),
                        tempo_map.frame_to_beat(*loop_end_frame),
                        tempo_map.frame_to_seconds(*loop_start_frame),
                        tempo_map.frame_to_seconds(*loop_end_frame),
                    ),
                };

            self.loop_state = loop_state.0;
            self.loop_start_beats = loop_start_beats;
            self.loop_end_beats = loop_end_beats;
            self.loop_start_seconds = loop_start_seconds;
            self.loop_end_seconds = loop_end_seconds;
        }

        // We don't need to return a new transport event if nothing has changed and
        // we are not currently playing.
        let do_return_event =
            self.is_playing || is_playing || tempo_map_changed || loop_state_changed;

        self.is_playing = is_playing;
        self.loop_back_info = None;
        self.playhead_frame = self.next_playhead_frame;
        if self.is_playing {
            // Advance the playhead.
            if let LoopState::Active { loop_start_frame, loop_end_frame } = self.loop_state {
                if self.playhead_frame < loop_end_frame
                    && self.playhead_frame + proc_frames >= loop_end_frame
                {
                    let first_frames = loop_end_frame - self.playhead_frame;
                    let second_frames = proc_frames - first_frames;

                    self.range_checker = RangeChecker::Looping {
                        end_frame_1: loop_end_frame,
                        start_frame_2: loop_start_frame,
                        end_frame_2: loop_start_frame + second_frames,
                    };

                    self.next_playhead_frame = loop_start_frame + second_frames;

                    self.loop_back_info = Some(LoopBackInfo {
                        loop_start: loop_start_frame,
                        loop_end: loop_end_frame,
                        playhead_end: self.next_playhead_frame,
                    });
                }
            }

            if self.loop_back_info.is_none() {
                self.next_playhead_frame = self.playhead_frame + proc_frames;

                self.range_checker = RangeChecker::Playing { end_frame: self.next_playhead_frame };
            }

            self.transport_info_at_frame = tempo_map.transport_info_at_frame(self.playhead_frame);
        } else {
            self.range_checker = RangeChecker::Paused;
        }

        self.playhead_frame_shared.store(self.next_playhead_frame, Ordering::Relaxed);

        let event: Option<TransportEvent> = if do_return_event {
            let song_pos_beats = tempo_map.frame_to_beat(self.playhead_frame);
            let song_pos_seconds = tempo_map.frame_to_seconds(self.playhead_frame);

            let mut transport_flags = TransportEventFlags::HAS_TEMPO
                | TransportEventFlags::HAS_BEATS_TIMELINE
                | TransportEventFlags::HAS_SECONDS_TIMELINE
                | TransportEventFlags::HAS_TIME_SIGNATURE;

            let tempo_inc = if self.is_playing {
                transport_flags |= TransportEventFlags::IS_PLAYING;
                self.transport_info_at_frame.tempo_inc
            } else {
                0.0
            };
            if let LoopState::Active { .. } = self.loop_state {
                transport_flags |= TransportEventFlags::IS_LOOP_ACTIVE
            }

            Some(TransportEvent {
                header: EventHeader::new_core(0, EventFlags::empty()),

                flags: transport_flags,

                song_pos_beats,
                song_pos_seconds,

                tempo: self.transport_info_at_frame.tempo,
                tempo_inc,

                loop_start_beats: self.loop_start_beats,
                loop_end_beats: self.loop_end_beats,
                loop_start_seconds: self.loop_start_seconds,
                loop_end_seconds: self.loop_end_seconds,

                bar_start: self.transport_info_at_frame.current_bar_start,
                bar_number: self.transport_info_at_frame.current_bar_number,

                time_signature_numerator: self.transport_info_at_frame.tsig_num as i16,
                time_signature_denominator: self.transport_info_at_frame.tsig_denom as i16,
            })
        } else {
            None
        };

        let jump_info = if let Some(info) = &self.loop_back_info {
            JumpInfo::Looped(info)
        } else if let Some(info) = &self.seek_info {
            JumpInfo::Seeked(info)
        } else {
            JumpInfo::None
        };

        self.declick.process(self.playhead_frame, frames, self.is_playing, jump_info);
        let declick_info = self.declick.get_info();

        TransportInfo::_new(
            self.playhead_frame,
            self.is_playing,
            self.loop_state,
            self.loop_back_info,
            self.seek_info,
            self.range_checker,
            event,
            declick_info,
        )
    }
}
