use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use meadowlark_plugin_api::buffer::SharedBuffer;
use meadowlark_plugin_api::ProcInfo;

use crate::plugin_host::event_io_buffers::NoteIoEvent;

pub(crate) struct NoteDelayCompTask {
    pub shared_node: SharedNoteDelayCompNode,

    pub note_in: SharedBuffer<NoteIoEvent>,
    pub note_out: SharedBuffer<NoteIoEvent>,
}

impl NoteDelayCompTask {
    pub fn process(&mut self, proc_info: &ProcInfo) {
        let mut delay_comp_node = self.shared_node.borrow_mut();

        delay_comp_node.process(proc_info, &self.note_in, &self.note_out);
    }
}

#[derive(Clone)]
pub(crate) struct SharedNoteDelayCompNode {
    pub active: bool,
    pub delay: u32,

    shared: Shared<AtomicRefCell<NoteDelayCompNode>>,
}

impl SharedNoteDelayCompNode {
    pub fn new(d: NoteDelayCompNode, coll_handle: &basedrop::Handle) -> Self {
        Self {
            active: true,
            delay: d.delay(),
            shared: Shared::new(coll_handle, AtomicRefCell::new(d)),
        }
    }

    pub fn borrow_mut(&self) -> AtomicRefMut<'_, NoteDelayCompNode> {
        self.shared.borrow_mut()
    }
}

pub(crate) struct NoteDelayCompNode {
    buf: Vec<NoteIoEvent>,
    temp_buf: Vec<NoteIoEvent>,
    delay: u32,
}

impl NoteDelayCompNode {
    pub fn new(delay: u32, note_buffer_size: usize) -> Self {
        Self {
            buf: Vec::with_capacity(note_buffer_size),
            temp_buf: Vec::with_capacity(note_buffer_size),
            delay,
        }
    }

    pub fn process(
        &mut self,
        proc_info: &ProcInfo,
        input: &SharedBuffer<NoteIoEvent>,
        output: &SharedBuffer<NoteIoEvent>,
    ) {
        let input_buf = input.borrow();
        let mut output_buf = output.borrow_mut();
        output_buf.data.clear();

        self.temp_buf.clear();

        for mut event in self.buf.drain(..) {
            if event.header.time < proc_info.frames as u32 {
                output_buf.data.push(event);
            } else {
                event.header.time -= proc_info.frames as u32;
                self.temp_buf.push(event);
            }
        }

        self.buf.append(&mut self.temp_buf);

        for event in input_buf.data.iter() {
            let mut event_delayed = *event;
            event_delayed.header.time += self.delay;

            if event_delayed.header.time < proc_info.frames as u32 {
                output_buf.data.push(event_delayed);
            } else {
                event_delayed.header.time -= proc_info.frames as u32;
                self.buf.push(event_delayed);
            }
        }
    }

    pub fn delay(&self) -> u32 {
        self.delay
    }
}
