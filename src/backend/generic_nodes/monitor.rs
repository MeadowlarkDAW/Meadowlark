use basedrop::{Handle, Shared};
use ringbuf::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::backend::graph::{clear_audio_outputs, AudioBlockBuffer, AudioGraphNode, ProcInfo};
use crate::backend::timeline::TimelineTransport;

pub struct MonoMonitorNodeHandle {
    pub monitor_rx: Consumer<f32>,
    active: Shared<AtomicBool>,
}

impl MonoMonitorNodeHandle {
    pub fn active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn set_active(&mut self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }
}

pub struct MonoMonitorNode {
    active: Shared<AtomicBool>,
    tx: Producer<f32>,
}

impl MonoMonitorNode {
    pub fn new(
        max_frames: usize,
        active: bool,
        coll_handle: &Handle,
    ) -> (Self, MonoMonitorNodeHandle) {
        let (tx, rx) = RingBuffer::<f32>::new(max_frames).split();

        let active = Shared::new(coll_handle, AtomicBool::new(active));

        (
            Self { active: Shared::clone(&active), tx },
            MonoMonitorNodeHandle { monitor_rx: rx, active },
        )
    }
}

impl AudioGraphNode for MonoMonitorNode {
    fn audio_in_ports(&self) -> usize {
        1
    }
    fn audio_out_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        audio_in: &[AudioBlockBuffer<f32>],
        audio_out: &mut [AudioBlockBuffer<f32>],
    ) {
        if audio_in.is_empty() {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            clear_audio_outputs(audio_out, proc_info);
            return;
        }

        let src = &audio_in[0];

        let frames = proc_info.frames();

        if self.active.load(Ordering::SeqCst) {
            self.tx.push_slice(&src.buf[0..frames]);
        }

        if audio_out.is_empty() {
            return;
        }

        let dst = &mut audio_out[0];

        dst.copy_frames_from(src, frames);
    }
}

pub struct StereoMonitorNodeHandle {
    pub monitor_left_rx: Consumer<f32>,
    pub monitor_right_rx: Consumer<f32>,
    active: Shared<AtomicBool>,
}

impl StereoMonitorNodeHandle {
    pub fn active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn set_active(&mut self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }
}

pub struct StereoMonitorNode {
    active: Shared<AtomicBool>,

    left_tx: Producer<f32>,
    right_tx: Producer<f32>,
}

impl StereoMonitorNode {
    pub fn new(
        max_frames: usize,
        active: bool,
        coll_handle: &Handle,
    ) -> (Self, StereoMonitorNodeHandle) {
        let (left_tx, left_rx) = RingBuffer::<f32>::new(max_frames).split();
        let (right_tx, right_rx) = RingBuffer::<f32>::new(max_frames).split();

        let active = Shared::new(coll_handle, AtomicBool::new(active));

        (
            Self { active: Shared::clone(&active), left_tx, right_tx },
            StereoMonitorNodeHandle {
                active,
                monitor_left_rx: left_rx,
                monitor_right_rx: right_rx,
            },
        )
    }
}

impl AudioGraphNode for StereoMonitorNode {
    fn audio_in_ports(&self) -> usize {
        2
    }
    fn audio_out_ports(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        proc_info: &ProcInfo,
        _transport: &TimelineTransport,
        audio_in: &[AudioBlockBuffer<f32>],
        audio_out: &mut [AudioBlockBuffer<f32>],
    ) {
        // Assume the host won't connect only one of the two channels in a stereo pair.
        if audio_in.len() < 2 {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            clear_audio_outputs(audio_out, proc_info);
            return;
        }

        let src_left = &audio_in[0];
        let src_right = &audio_in[1];

        let frames = proc_info.frames();

        if self.active.load(Ordering::SeqCst) {
            self.left_tx.push_slice(&src_left.buf[0..frames]);
            self.right_tx.push_slice(&src_right.buf[0..frames]);
        }

        // Assume the host won't connect only one of the two channels in a stereo pair.
        if audio_out.len() < 2 {
            // As per the spec, all unused audio output buffers must be cleared to 0.0.
            clear_audio_outputs(audio_out, proc_info);
            return;
        }

        let dst_left = &mut audio_out[0];
        let dst_right = &mut audio_out[1];

        dst_left.copy_frames_from(src_left, frames);
        dst_right.copy_frames_from(src_right, frames);
    }
}
