use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use basedrop::{Shared, SharedCell};
use clack_host::events::{Event, EventFlags, EventHeader};
use clack_host::utils::Cookie;
use meadowlark_plugin_api::{
    buffer::EventBuffer,
    event::{ParamModEvent, ParamValueEvent},
    ParamID, PluginProcessor,
};
use std::sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
};

use meadowlark_plugin_api::automation::AutomationIoEventType;

use crate::utils::reducing_queue::{
    ReducFnvConsumer, ReducFnvProducer, ReducFnvValue, ReducingFnvQueue,
};
use crate::utils::thread_id::SharedThreadIDs;

use super::processor::PluginHostProcessor;

pub(super) struct PlugHostChannelMainThread {
    pub param_queues: Option<ParamQueuesMainThread>,
    pub shared_state: Arc<SharedPluginHostState>,

    shared_processor: SharedPluginHostProcessor,
}

impl PlugHostChannelMainThread {
    pub fn new(bypassed: bool, coll_handle: &basedrop::Handle) -> Self {
        Self {
            param_queues: None,
            shared_processor: SharedPluginHostProcessor::new(None, coll_handle),
            shared_state: Arc::new(SharedPluginHostState::new(bypassed)),
        }
    }

    /// Send the new processor to the process thread.
    #[allow(clippy::too_many_arguments)] // Fix this?
    pub fn new_processor(
        &mut self,
        plugin_processor: Box<dyn PluginProcessor>,
        plugin_instance_id: u64,
        num_params: usize,
        thread_ids: SharedThreadIDs,
        schedule_version: u64,
        bypass_declick_frames: usize,
        coll_handle: &basedrop::Handle,
    ) {
        let (param_queues_main_thread, param_queues_proc_thread) = if num_params > 0 {
            let (main_to_proc_param_value_tx, main_to_proc_param_value_rx) =
                ReducingFnvQueue::new_channel(num_params, coll_handle);
            let (main_to_proc_param_mod_tx, main_to_proc_param_mod_rx) =
                ReducingFnvQueue::new_channel(num_params, coll_handle);
            let (proc_to_main_param_value_tx, proc_to_main_param_value_rx) =
                ReducingFnvQueue::new_channel(num_params, coll_handle);

            (
                Some(ParamQueuesMainThread {
                    to_proc_param_value_tx: main_to_proc_param_value_tx,
                    to_proc_param_mod_tx: main_to_proc_param_mod_tx,
                    from_proc_param_value_rx: proc_to_main_param_value_rx,
                }),
                Some(ParamQueuesProcThread {
                    from_main_param_value_rx: main_to_proc_param_value_rx,
                    from_main_param_mod_rx: main_to_proc_param_mod_rx,
                    to_main_param_value_tx: proc_to_main_param_value_tx,
                }),
            )
        } else {
            (None, None)
        };

        self.param_queues = param_queues_main_thread;

        let proc_channel = PlugHostChannelProcThread {
            param_queues: param_queues_proc_thread,
            shared_state: Arc::clone(&self.shared_state),
        };

        self.shared_processor.set(
            PluginHostProcessor::new(
                plugin_processor,
                plugin_instance_id,
                proc_channel,
                num_params,
                thread_ids,
                schedule_version,
                bypass_declick_frames,
            ),
            coll_handle,
        );
    }

    pub fn drop_processor_pointer_on_main_thread(
        &mut self,
        coll_handle: &basedrop::Handle,
    ) -> Shared<PluginHostProcessorWrapper> {
        self.param_queues = None;
        self.shared_processor.remove(coll_handle)
    }

    pub fn shared_processor(&self) -> &SharedPluginHostProcessor {
        &self.shared_processor
    }
}

pub(crate) struct PlugHostChannelProcThread {
    pub param_queues: Option<ParamQueuesProcThread>,
    pub shared_state: Arc<SharedPluginHostState>,
}

pub(super) struct ParamQueuesMainThread {
    pub to_proc_param_value_tx: ReducFnvProducer<ParamID, MainToProcParamValue>,
    pub to_proc_param_mod_tx: ReducFnvProducer<ParamID, MainToProcParamValue>,

    pub from_proc_param_value_rx: ReducFnvConsumer<ParamID, ProcToMainParamValue>,
}

pub(crate) struct ParamQueuesProcThread {
    pub from_main_param_value_rx: ReducFnvConsumer<ParamID, MainToProcParamValue>,
    pub from_main_param_mod_rx: ReducFnvConsumer<ParamID, MainToProcParamValue>,

    pub to_main_param_value_tx: ReducFnvProducer<ParamID, ProcToMainParamValue>,
}

impl ParamQueuesProcThread {
    pub fn consume_into_event_buffer(&mut self, buffer: &mut EventBuffer) -> bool {
        let mut has_param_in_event = false;
        self.from_main_param_value_rx.consume(|param_id, value| {
            has_param_in_event = true;

            let event = ParamValueEvent::new(
                // TODO: Finer values for `time` instead of just setting it to the first frame?
                EventHeader::new_core(0, EventFlags::empty()),
                value.cookie,
                // TODO: Note ID
                -1,                // note_id
                param_id.as_u32(), // param_id
                // TODO: Port index
                -1, // port_index
                // TODO: Channel
                -1, // channel
                // TODO: Key
                -1,          // key
                value.value, // value
            );

            buffer.push(event.as_unknown())
        });

        self.from_main_param_mod_rx.consume(|param_id, value| {
            has_param_in_event = true;

            let event = ParamModEvent::new(
                // TODO: Finer values for `time` instead of just setting it to the first frame?
                EventHeader::new_core(0, EventFlags::empty()),
                value.cookie,
                // TODO: Note ID
                -1,                // note_id
                param_id.as_u32(), // param_id
                // TODO: Port index
                -1, // port_index
                // TODO: Channel
                -1, // channel
                // TODO: Key
                -1,          // key
                value.value, // value
            );

            buffer.push(event.as_unknown())
        });
        has_param_in_event
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MainToProcParamValue {
    pub value: f64,
    pub cookie: Cookie,
}

impl ReducFnvValue for MainToProcParamValue {}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParamGestureInfo {
    pub is_begin: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ProcToMainParamValue {
    pub value: Option<f64>,
    pub gesture: Option<ParamGestureInfo>,
}

impl ReducFnvValue for ProcToMainParamValue {
    fn update(&mut self, new_value: &Self) {
        if new_value.value.is_some() {
            self.value = new_value.value;
        }

        if new_value.gesture.is_some() {
            self.gesture = new_value.gesture;
        }
    }
}

impl ProcToMainParamValue {
    pub fn from_param_event(event: AutomationIoEventType) -> Option<Self> {
        match event {
            AutomationIoEventType::Value(value) => Some(Self { value: Some(value), gesture: None }),
            AutomationIoEventType::Modulation(_) => None, // TODO: handle mod events
            AutomationIoEventType::BeginGesture => {
                Some(Self { value: None, gesture: Some(ParamGestureInfo { is_begin: true }) })
            }
            AutomationIoEventType::EndGesture => {
                Some(Self { value: None, gesture: Some(ParamGestureInfo { is_begin: false }) })
            }
        }
    }
}

pub(crate) struct SharedPluginHostState {
    active_state: AtomicU32,
    process_requested: AtomicBool,
    param_flush_requested: AtomicBool,
    bypassed: AtomicBool,
}

impl SharedPluginHostState {
    pub fn new(bypassed: bool) -> Self {
        Self {
            active_state: AtomicU32::new(0),
            process_requested: AtomicBool::new(false),
            param_flush_requested: AtomicBool::new(false),
            bypassed: AtomicBool::new(bypassed),
        }
    }

    pub fn get_active_state(&self) -> PluginActiveState {
        let s = self.active_state.load(Ordering::SeqCst);
        s.into()
    }

    pub fn set_active_state(&self, state: PluginActiveState) {
        self.active_state.store(state as u32, Ordering::SeqCst);
    }

    pub fn process_requested(&self) -> bool {
        self.process_requested.swap(false, Ordering::SeqCst)
    }

    pub fn set_process_requested(&self) {
        self.process_requested.store(true, Ordering::SeqCst)
    }

    pub fn param_flush_requested(&self) -> bool {
        self.param_flush_requested.swap(false, Ordering::SeqCst)
    }

    pub fn set_param_flush_requested(&self) {
        self.param_flush_requested.store(true, Ordering::SeqCst)
    }

    pub fn bypassed(&self) -> bool {
        self.bypassed.load(Ordering::SeqCst)
    }

    pub fn set_bypassed(&self, bypassed: bool) {
        self.bypassed.store(bypassed, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub(crate) enum PluginActiveState {
    // TODO: this state shouldn't be able to exist for the process thread
    /// The plugin is inactive, only the main thread uses it.
    Inactive = 0,

    /// Activation failed.
    InactiveWithError = 1,

    /// The plugin is active. It may or may not be processing right now.
    Active = 2,

    /// The main thread is waiting for the process thread to drop the plugin's audio processor.
    WaitingToDrop = 3,

    /// The plugin is not used anymore by the audio engine and can be deactivated on the main
    /// thread.
    DroppedAndReadyToDeactivate = 4,
}

impl PluginActiveState {
    pub fn is_active(&self) -> bool {
        !(*self == PluginActiveState::Inactive || *self == PluginActiveState::InactiveWithError)
    }
}

impl From<u32> for PluginActiveState {
    fn from(s: u32) -> Self {
        match s {
            0 => PluginActiveState::Inactive,
            1 => PluginActiveState::InactiveWithError,
            2 => PluginActiveState::Active,
            3 => PluginActiveState::WaitingToDrop,
            4 => PluginActiveState::DroppedAndReadyToDeactivate,
            _ => PluginActiveState::InactiveWithError,
        }
    }
}

pub(crate) struct PluginHostProcessorWrapper {
    processor: AtomicRefCell<Option<PluginHostProcessor>>,
}

impl PluginHostProcessorWrapper {
    pub fn borrow_mut(&self) -> AtomicRefMut<'_, Option<PluginHostProcessor>> {
        self.processor.borrow_mut()
    }
}

unsafe impl Send for PluginHostProcessorWrapper {}
unsafe impl Sync for PluginHostProcessorWrapper {}

#[derive(Clone)]
pub(crate) struct SharedPluginHostProcessor {
    shared: Shared<SharedCell<PluginHostProcessorWrapper>>,
}

impl SharedPluginHostProcessor {
    pub fn new(p: Option<PluginHostProcessor>, coll_handle: &basedrop::Handle) -> Self {
        Self {
            shared: Shared::new(
                coll_handle,
                SharedCell::new(Shared::new(
                    coll_handle,
                    PluginHostProcessorWrapper { processor: AtomicRefCell::new(p) },
                )),
            ),
        }
    }

    fn set(&mut self, p: PluginHostProcessor, coll_handle: &basedrop::Handle) {
        self.shared.set(Shared::new(
            coll_handle,
            PluginHostProcessorWrapper { processor: AtomicRefCell::new(Some(p)) },
        ))
    }

    fn remove(&mut self, coll_handle: &basedrop::Handle) -> Shared<PluginHostProcessorWrapper> {
        self.shared.replace(Shared::new(
            coll_handle,
            PluginHostProcessorWrapper { processor: AtomicRefCell::new(None) },
        ))
    }

    pub fn get(&self) -> Shared<PluginHostProcessorWrapper> {
        self.shared.get()
    }
}
