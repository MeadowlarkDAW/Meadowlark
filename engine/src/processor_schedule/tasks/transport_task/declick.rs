use atomic_refcell::AtomicRefCell;
use basedrop::Shared;
use meadowlark_plugin_api::transport::{DeclickBuffers, DeclickInfo, LoopBackInfo, SeekInfo};

pub(super) enum JumpInfo<'a> {
    None,
    Seeked(&'a SeekInfo),
    Looped(&'a LoopBackInfo),
}

#[derive(Clone, Copy)]
enum JumpDeclickState {
    NotDeclicking,
    DeclickingSeek {
        out_val: f32,
        in_val: f32,

        frames_left: usize,
    },
    DeclickingLoop {
        out_val: f32,
        in_val: f32,

        skip_frames: usize,

        in_frames_left: usize,
        out_frames_left: usize,
    },
}

#[derive(Clone)]
pub(super) struct TransportDeclick {
    buffers: Shared<AtomicRefCell<DeclickBuffers>>,

    start_stop_active: bool,
    start_stop_buf_state: Option<(f32, usize)>,

    jump_active: bool,
    jump_state: JumpDeclickState,

    jump_in_playhead_frame: i64,
    jump_out_playhead_frame: u64,
    jump_in_playhead_next_frame: i64,
    jump_out_playhead_next_frame: u64,

    start_declick_start_frame: u64,
    jump_in_declick_start_frame: i64,

    declick_frames: usize,
    declick_inc: f32,

    is_playing: bool,
}

impl TransportDeclick {
    pub fn new(
        max_frames: usize,
        declick_seconds: f64,
        sample_rate: u32,
        coll_handle: &basedrop::Handle,
    ) -> Self {
        assert!(declick_seconds > 0.0);

        let declick_frames = (declick_seconds * f64::from(sample_rate)).round() as usize;
        let declick_inc = 1.0 / declick_frames as f32;

        let buffers = Shared::new(
            coll_handle,
            AtomicRefCell::new(DeclickBuffers {
                start_stop_buf: vec![0.0; max_frames],
                jump_out_buf: vec![0.0; max_frames],
                jump_in_buf: vec![0.0; max_frames],
            }),
        );

        Self {
            buffers,

            start_stop_active: false,
            start_stop_buf_state: None,

            jump_active: false,
            jump_state: JumpDeclickState::NotDeclicking,

            jump_in_playhead_frame: 0,
            jump_out_playhead_frame: 0,
            jump_in_playhead_next_frame: 0,
            jump_out_playhead_next_frame: 0,

            start_declick_start_frame: 0,
            jump_in_declick_start_frame: 0,

            declick_frames,
            declick_inc,

            is_playing: false,
        }
    }

    pub fn process(
        &mut self,
        playhead_frame: u64,
        frames: usize,
        is_playing: bool,
        jump_info: JumpInfo<'_>,
    ) {
        self.start_stop_active = false;
        self.jump_active = false;

        if is_playing != self.is_playing {
            self.is_playing = is_playing;

            if is_playing {
                // Transport just started.
                self.start_stop_buf_state =
                    if let Some((val, frames_left)) = self.start_stop_buf_state {
                        // If the "stop" declick was still running, find the number of frames
                        // needed to bring it back up to 1.0.
                        if frames_left < self.declick_frames {
                            Some((val, self.declick_frames - frames_left))
                        } else {
                            None
                        }
                    } else {
                        Some((0.0, self.declick_frames))
                    };

                self.start_declick_start_frame = playhead_frame;
            } else {
                // Transport just stopped.
                self.start_stop_buf_state =
                    if let Some((val, frames_left)) = self.start_stop_buf_state {
                        // If the "start" declick was still running, find the number of frames
                        // needed to bring it back down to 0.0.
                        if frames_left < self.declick_frames {
                            Some((val, self.declick_frames - frames_left))
                        } else {
                            None
                        }
                    } else {
                        Some((1.0, self.declick_frames))
                    };
            }
        }

        match jump_info {
            JumpInfo::Seeked(info) => {
                if self.start_stop_buf_state.is_some() {
                    // The transport just seeked to a new position while playing.

                    self.jump_state = JumpDeclickState::DeclickingSeek {
                        out_val: 1.0,
                        in_val: 0.0,

                        frames_left: self.declick_frames,
                    };
                    self.jump_in_playhead_next_frame = playhead_frame as i64;
                    self.jump_out_playhead_next_frame = info.seeked_from_playhead;

                    self.jump_in_declick_start_frame = self.jump_in_playhead_next_frame;
                }
            }
            JumpInfo::Looped(info) => {
                // The transport is looping this process cycle.

                let skip_frames = info.loop_end - playhead_frame;

                self.jump_state = JumpDeclickState::DeclickingLoop {
                    out_val: 1.0,
                    in_val: 0.0,

                    in_frames_left: self.declick_frames,
                    out_frames_left: self.declick_frames,

                    skip_frames: skip_frames as usize,
                };
                self.jump_in_playhead_next_frame = info.loop_start as i64 - skip_frames as i64;
                self.jump_out_playhead_next_frame = playhead_frame;

                self.jump_in_declick_start_frame = self.jump_in_playhead_next_frame;
            }
            _ => {}
        }

        let mut buffers = self.buffers.borrow_mut();
        let DeclickBuffers { start_stop_buf, jump_out_buf, jump_in_buf } = &mut *buffers;

        if let Some((mut val, frames_left)) = self.start_stop_buf_state {
            self.start_stop_active = true;

            let (declick_frames, constant_frames) = if frames_left >= frames {
                (frames, 0)
            } else {
                (frames_left, frames - frames_left)
            };

            let buf = &mut start_stop_buf[0..frames];

            // Fill the buffer with declick frames.
            let buf_part = &mut buf[0..declick_frames];
            if is_playing {
                for s in buf_part.iter_mut() {
                    *s = val;
                    val += self.declick_inc;
                }
            } else {
                for s in buf_part.iter_mut() {
                    *s = val;
                    val -= self.declick_inc;
                }
            }

            // Fill the rest with a constant if the end of the declick has been reached.
            if constant_frames > 0 {
                if is_playing {
                    buf[declick_frames..frames].fill(1.0);
                } else {
                    buf[declick_frames..frames].fill(0.0);
                }
            }

            self.start_stop_buf_state =
                if frames_left > frames { Some((val, frames_left - frames)) } else { None };
        }

        match self.jump_state {
            JumpDeclickState::DeclickingSeek { mut out_val, mut in_val, frames_left } => {
                self.jump_active = true;

                let (declick_frames, constant_frames) = if frames_left >= frames {
                    (frames, 0)
                } else {
                    (frames_left, frames - frames_left)
                };

                let out_buf = &mut jump_out_buf[0..frames];
                let in_buf = &mut jump_in_buf[0..frames];

                // Fill the buffers with declick frames.
                let out_buf_part_1 = &mut out_buf[0..declick_frames];
                let in_buf_part_1 = &mut in_buf[0..declick_frames];
                for i in 0..declick_frames {
                    out_buf_part_1[i] = out_val;
                    in_buf_part_1[i] = in_val;

                    out_val -= self.declick_inc;
                    in_val += self.declick_inc;
                }

                // Fill the rest with a constant if the end of the declick has been reached.
                if constant_frames > 0 {
                    out_buf[declick_frames..frames].fill(0.0);
                    in_buf[declick_frames..frames].fill(1.0);
                }

                self.jump_in_playhead_frame = self.jump_in_playhead_next_frame;
                self.jump_out_playhead_frame = self.jump_out_playhead_next_frame;
                self.jump_in_playhead_next_frame += frames as i64;
                self.jump_out_playhead_next_frame += frames as u64;

                self.jump_state = if frames_left > frames {
                    JumpDeclickState::DeclickingSeek {
                        out_val,
                        in_val,
                        frames_left: frames_left - frames,
                    }
                } else {
                    JumpDeclickState::NotDeclicking
                };
            }
            JumpDeclickState::DeclickingLoop {
                mut out_val,
                mut in_val,
                skip_frames,
                mut in_frames_left,
                mut out_frames_left,
            } => {
                self.jump_active = true;

                let out_buf = &mut jump_out_buf[0..frames];
                let in_buf = &mut jump_in_buf[0..frames];

                if skip_frames >= frames {
                    out_buf.fill(1.0);
                    in_buf.fill(0.0);

                    self.jump_state = JumpDeclickState::DeclickingLoop {
                        out_val,
                        in_val,
                        skip_frames: skip_frames - frames,
                        in_frames_left,
                        out_frames_left,
                    };
                    self.jump_in_playhead_frame = self.jump_in_playhead_next_frame;
                    self.jump_out_playhead_frame = self.jump_out_playhead_next_frame;
                    self.jump_in_playhead_next_frame += frames as i64;
                    self.jump_out_playhead_next_frame += frames as u64;

                    return;
                } else if skip_frames > 0 {
                    out_buf[0..skip_frames].fill(1.0);
                    in_buf[0..skip_frames].fill(0.0);
                }

                let out_buf = &mut out_buf[skip_frames..frames];
                let in_buf = &mut in_buf[skip_frames..frames];

                let frames_left = frames - skip_frames;

                if out_frames_left > 0 {
                    let (out_declick_frames, out_constant_frames) =
                        if out_frames_left >= frames_left {
                            (frames_left, 0)
                        } else {
                            (out_frames_left, frames_left - out_frames_left)
                        };

                    // Fill the out buffer with declick frames.
                    let out_buf_part = &mut out_buf[0..out_declick_frames];
                    for v in out_buf_part.iter_mut() {
                        *v = in_val;
                        out_val -= self.declick_inc;
                    }

                    // Fill the rest with a constant if the end of the declick has been reached.
                    if out_constant_frames > 0 {
                        out_buf[out_declick_frames..frames_left].fill(0.0);
                    }

                    if out_frames_left > frames_left {
                        out_frames_left -= frames_left;
                    } else {
                        out_frames_left = 0;
                    }
                } else {
                    out_buf.fill(0.0);
                }

                if in_frames_left > 0 {
                    let (in_declick_frames, in_constant_frames) = if in_frames_left >= frames_left {
                        (frames_left, 0)
                    } else {
                        (in_frames_left, frames_left - in_frames_left)
                    };

                    // Fill the in buffer with declick frames.

                    let in_buf_part = &mut in_buf[0..in_declick_frames];
                    for v in in_buf_part.iter_mut() {
                        *v = in_val;
                        in_val += self.declick_inc;
                    }

                    // Fill the rest with a constant if the end of the declick has been reached.
                    if in_constant_frames > 0 {
                        in_buf[in_declick_frames..frames_left].fill(1.0);
                    }

                    if in_frames_left > frames_left {
                        in_frames_left -= frames_left;
                    } else {
                        in_frames_left = 0;
                    }
                } else {
                    in_buf.fill(1.0);
                }

                self.jump_in_playhead_frame = self.jump_in_playhead_next_frame;
                self.jump_out_playhead_frame = self.jump_out_playhead_next_frame;
                self.jump_in_playhead_next_frame += frames as i64;
                self.jump_out_playhead_next_frame += frames as u64;

                self.jump_state = if in_frames_left > 0 || out_frames_left > 0 {
                    JumpDeclickState::DeclickingLoop {
                        out_val,
                        in_val,
                        skip_frames: 0,
                        in_frames_left,
                        out_frames_left,
                    }
                } else {
                    JumpDeclickState::NotDeclicking
                };
            }
            _ => {}
        }
    }

    pub fn get_info(&self) -> DeclickInfo {
        DeclickInfo::_new(
            Shared::clone(&self.buffers),
            self.start_stop_active,
            self.jump_active,
            self.jump_in_playhead_frame,
            self.jump_out_playhead_frame,
            self.start_declick_start_frame,
            self.jump_in_declick_start_frame,
        )
    }
}
