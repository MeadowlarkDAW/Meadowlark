mod buffer_pool;
mod delay_comp_node_pool;
mod plugin_host_pool;
mod shared_schedule;
mod transport_pool;

pub(crate) use buffer_pool::SharedBufferPool;
pub(crate) use delay_comp_node_pool::{DelayCompKey, DelayCompNodePool};
pub(crate) use plugin_host_pool::PluginHostPool;
pub(crate) use shared_schedule::SharedProcessorSchedule;
pub(crate) use transport_pool::{SharedTransportTask, TransportPool};

use crate::{
    processor_schedule::{tasks::TransportTask, ProcessorSchedule},
    utils::thread_id::SharedThreadIDs,
};

pub(super) struct GraphSharedPools {
    pub shared_schedule: SharedProcessorSchedule,

    pub buffers: SharedBufferPool,
    pub plugin_hosts: PluginHostPool,
    pub delay_comp_nodes: DelayCompNodePool,
    pub transports: TransportPool,
}

impl GraphSharedPools {
    pub fn new(
        thread_ids: SharedThreadIDs,
        audio_buffer_size: usize,
        note_buffer_size: usize,
        event_buffer_size: usize,
        transport: TransportTask,
        schedule_version: u64,
        coll_handle: basedrop::Handle,
    ) -> (Self, SharedProcessorSchedule) {
        let shared_transport_task = SharedTransportTask::new(transport, &coll_handle);

        let empty_schedule = ProcessorSchedule::new_empty(
            audio_buffer_size,
            shared_transport_task.clone(),
            Vec::new(),
            schedule_version,
        );

        let (shared_schedule, shared_schedule_clone) =
            SharedProcessorSchedule::new(empty_schedule, thread_ids, &coll_handle);

        (
            Self {
                shared_schedule,
                buffers: SharedBufferPool::new(
                    audio_buffer_size,
                    note_buffer_size,
                    event_buffer_size,
                    coll_handle,
                ),
                plugin_hosts: PluginHostPool::new(),
                delay_comp_nodes: DelayCompNodePool::new(),
                transports: TransportPool { transport: shared_transport_task },
            },
            shared_schedule_clone,
        )
    }
}
