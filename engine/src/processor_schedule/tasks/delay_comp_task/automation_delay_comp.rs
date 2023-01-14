use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use meadowlark_plugin_api::automation::AutomationIoEvent;
use meadowlark_plugin_api::buffer::SharedBuffer;
use meadowlark_plugin_api::ProcInfo;

pub(crate) struct AutomationDelayCompTask {
    pub shared_node: SharedAutomationDelayCompNode,

    pub input: SharedBuffer<AutomationIoEvent>,
    pub output: SharedBuffer<AutomationIoEvent>,
}

impl AutomationDelayCompTask {
    pub fn process(&mut self, proc_info: &ProcInfo) {
        let mut delay_comp_node = self.shared_node.borrow_mut();

        delay_comp_node.process(proc_info, &self.input, &self.output);
    }
}

#[derive(Clone)]
pub(crate) struct SharedAutomationDelayCompNode {
    pub active: bool,
    pub delay: u32,

    shared: Shared<AtomicRefCell<AutomationDelayCompNode>>,
}

impl SharedAutomationDelayCompNode {
    pub fn new(d: AutomationDelayCompNode, coll_handle: &basedrop::Handle) -> Self {
        Self {
            active: true,
            delay: d.delay(),
            shared: Shared::new(coll_handle, AtomicRefCell::new(d)),
        }
    }

    pub fn borrow_mut(&self) -> AtomicRefMut<'_, AutomationDelayCompNode> {
        self.shared.borrow_mut()
    }
}

pub(crate) struct AutomationDelayCompNode {
    buf: Vec<AutomationIoEvent>,
    temp_buf: Vec<AutomationIoEvent>,
    delay: u32,
}

impl AutomationDelayCompNode {
    pub fn new(delay: u32, automation_buffer_size: usize) -> Self {
        Self {
            buf: Vec::with_capacity(automation_buffer_size),
            temp_buf: Vec::with_capacity(automation_buffer_size),
            delay,
        }
    }

    pub fn process(
        &mut self,
        proc_info: &ProcInfo,
        input: &SharedBuffer<AutomationIoEvent>,
        output: &SharedBuffer<AutomationIoEvent>,
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
