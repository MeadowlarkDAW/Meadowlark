use basedrop::{Handle, Shared};
use ringbuf::{Consumer, Producer, RingBuffer};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::shared_state::{AudioGraphNode, ProcInfo};

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
            Self {
                active: Shared::clone(&active),
                tx,
            },
            MonoMonitorNodeHandle {
                monitor_rx: rx,
                active,
            },
        )
    }
}

impl AudioGraphNode for MonoMonitorNode {
    fn audio_through_ports(&self) -> usize {
        0
    }
    fn extra_audio_in_ports(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        _proc_info: &ProcInfo,
        _audio_through: &mut Vec<Shared<Vec<f32>>>,
        extra_audio_in: &Vec<Shared<Vec<f32>>>,
        _extra_audio_out: &mut Vec<Shared<Vec<f32>>>,
    ) {
        if self.active.load(Ordering::SeqCst) {
            self.tx.push_slice(&extra_audio_in[0]);
        }
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
            Self {
                active: Shared::clone(&active),
                left_tx,
                right_tx,
            },
            StereoMonitorNodeHandle {
                active,
                monitor_left_rx: left_rx,
                monitor_right_rx: right_rx,
            },
        )
    }
}

impl AudioGraphNode for StereoMonitorNode {
    fn audio_through_ports(&self) -> usize {
        0
    }
    fn extra_audio_in_ports(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        _proc_info: &ProcInfo,
        _audio_through: &mut Vec<Shared<Vec<f32>>>,
        extra_audio_in: &Vec<Shared<Vec<f32>>>,
        _extra_audio_out: &mut Vec<Shared<Vec<f32>>>,
    ) {
        if self.active.load(Ordering::SeqCst) {
            self.left_tx.push_slice(&extra_audio_in[0]);
            self.right_tx.push_slice(&extra_audio_in[1]);
        }
    }
}
