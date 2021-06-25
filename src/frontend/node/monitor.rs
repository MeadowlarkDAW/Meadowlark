use ringbuf::{Consumer, Producer, RingBuffer};

use super::{AudioGraphNode, ProcInfo};

pub struct MonoMonitorNode {
    pub active: bool,

    tx: Producer<f32>,
}

impl MonoMonitorNode {
    pub fn new(max_frames: usize, active: bool) -> (Self, Consumer<f32>) {
        let (tx, rx) = RingBuffer::<f32>::new(max_frames).split();

        (Self { active, tx }, rx)
    }
}

impl AudioGraphNode for MonoMonitorNode {
    fn audio_through_ports(&self) -> usize {
        0
    }
    fn extra_audio_in_ports(&self) -> usize {
        1
    }

    fn process(&mut self, proc_info: ProcInfo) {
        if self.active {
            self.tx.push_slice(&proc_info.extra_audio_in[0]);
        }
    }
}

pub struct StereoMonitorNode {
    pub active: bool,

    left_tx: Producer<f32>,
    right_tx: Producer<f32>,
}

impl StereoMonitorNode {
    pub fn new(max_frames: usize, active: bool) -> (Self, Consumer<f32>, Consumer<f32>) {
        let (left_tx, left_rx) = RingBuffer::<f32>::new(max_frames).split();
        let (right_tx, right_rx) = RingBuffer::<f32>::new(max_frames).split();

        (
            Self {
                active,
                left_tx,
                right_tx,
            },
            left_rx,
            right_rx,
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

    fn process(&mut self, proc_info: ProcInfo) {
        if self.active {
            self.left_tx.push_slice(&proc_info.extra_audio_in[0]);
            self.right_tx.push_slice(&proc_info.extra_audio_in[1]);
        }
    }
}
