use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use meadowlark_plugin_api::buffer::SharedBuffer;
use meadowlark_plugin_api::ProcInfo;

pub(crate) struct AudioDelayCompTask {
    pub shared_node: SharedAudioDelayCompNode,

    pub audio_in: SharedBuffer<f32>,
    pub audio_out: SharedBuffer<f32>,
}

impl AudioDelayCompTask {
    pub fn process(&mut self, proc_info: &ProcInfo) {
        let mut delay_comp_node = self.shared_node.borrow_mut();

        delay_comp_node.process(proc_info, &self.audio_in, &self.audio_out);
    }
}

#[derive(Clone)]
pub(crate) struct SharedAudioDelayCompNode {
    pub active: bool,
    pub delay: u32,

    shared: Shared<AtomicRefCell<AudioDelayCompNode>>,
}

impl SharedAudioDelayCompNode {
    pub fn new(d: AudioDelayCompNode, coll_handle: &basedrop::Handle) -> Self {
        Self {
            active: true,
            delay: d.delay(),
            shared: Shared::new(coll_handle, AtomicRefCell::new(d)),
        }
    }

    pub fn borrow_mut(&self) -> AtomicRefMut<'_, AudioDelayCompNode> {
        self.shared.borrow_mut()
    }
}

pub(crate) struct AudioDelayCompNode {
    buf: Vec<f32>,
    read_pointer: usize,
}

impl AudioDelayCompNode {
    pub fn new(delay: u32) -> Self {
        Self { buf: vec![0.0; delay as usize], read_pointer: 0 }
    }

    pub fn process(
        &mut self,
        proc_info: &ProcInfo,
        input: &SharedBuffer<f32>,
        output: &SharedBuffer<f32>,
    ) {
        let (input_ref, mut output_ref) = (input.borrow(), output.borrow_mut());

        let (in_buf, out_buf) =
            (&input_ref.data[0..proc_info.frames], &mut output_ref.data[0..proc_info.frames]);

        if proc_info.frames > self.buf.len() {
            if self.read_pointer == 0 {
                // Only one copy operation is needed.

                // Copy all frames from self.buf into the output buffer.
                out_buf[0..self.buf.len()].copy_from_slice(&self.buf[0..self.buf.len()]);
            } else if self.read_pointer < self.buf.len() {
                // This check will always be true, it is here to hint to the compiler to optimize.
                // Two copy operations are needed.

                let first_len = self.buf.len() - self.read_pointer;

                // Copy frames from self.buf into the output buffer.
                out_buf[0..first_len].copy_from_slice(&self.buf[self.read_pointer..self.buf.len()]);
                out_buf[first_len..self.buf.len()].copy_from_slice(&self.buf[0..self.read_pointer]);
            }

            // Copy the remaining frames from the input buffer to the output buffer.
            let remaining = proc_info.frames - self.buf.len();
            out_buf[self.buf.len()..proc_info.frames].copy_from_slice(&in_buf[0..remaining]);

            // Copy the final remaining frames from the input buffer into self.buf.
            // self.buf is "empty" at this point, so reset the read pointer so only one copy operation is needed.
            self.read_pointer = 0;
            let buf_len = self.buf.len();
            self.buf[0..buf_len].copy_from_slice(&in_buf[remaining..proc_info.frames]);
        } else {
            if self.read_pointer + proc_info.frames <= self.buf.len() {
                // Only one copy operation is needed.

                // Copy frames from self.buf into the output buffer.
                out_buf[0..proc_info.frames].copy_from_slice(
                    &self.buf[self.read_pointer..self.read_pointer + proc_info.frames],
                );

                // Copy all frames from the input buffer into self.buf.
                self.buf[self.read_pointer..self.read_pointer + proc_info.frames]
                    .copy_from_slice(&in_buf[0..proc_info.frames]);
            } else {
                // Two copy operations are needed.

                let first_len = self.buf.len() - self.read_pointer;
                let second_len = proc_info.frames - first_len;

                // Copy frames from self.buf into the output buffer.
                out_buf[0..first_len].copy_from_slice(&self.buf[self.read_pointer..self.buf.len()]);
                out_buf[first_len..proc_info.frames].copy_from_slice(&self.buf[0..second_len]);

                // Copy all frames from the input buffer into self.buf.
                let buf_len = self.buf.len();
                self.buf[self.read_pointer..buf_len].copy_from_slice(&in_buf[0..first_len]);
                self.buf[0..second_len].copy_from_slice(&in_buf[first_len..proc_info.frames]);
            }

            // Get the next position of the read pointer.
            self.read_pointer += proc_info.frames;
            if self.read_pointer >= self.buf.len() {
                self.read_pointer -= self.buf.len();
            }
        }

        // TODO: More efficient way to check if the output is constant?
        let mut is_constant = true;
        let val = out_buf[0];
        for x in out_buf.iter().skip(1) {
            if *x != val {
                is_constant = false;
                break;
            }
        }

        output_ref.is_constant = is_constant;
    }

    pub fn delay(&self) -> u32 {
        self.buf.len() as u32
    }
}
