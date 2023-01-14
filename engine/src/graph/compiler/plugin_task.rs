use audio_graph::ScheduledNode;
use fnv::FnvHashMap;
use meadowlark_plugin_api::automation::AutomationIoEvent;
use meadowlark_plugin_api::buffer::SharedBuffer;

use crate::plugin_host::event_io_buffers::NoteIoEvent;
use crate::processor_schedule::tasks::Task;

use super::super::error::GraphCompilerError;
use super::super::shared_pools::GraphSharedPools;
use super::super::{PortChannelID, PortType};

mod loaded_plugin_task;
mod unloaded_plugin_task;

pub(super) fn construct_plugin_task(
    scheduled_node: &ScheduledNode,
    shared_pool: &mut GraphSharedPools,
) -> Result<Task, GraphCompilerError> {
    // --- Get port info and processor from the plugin host ---------------------------------

    let plugin_host =
        shared_pool.plugin_hosts.get_by_node_id(&scheduled_node.id).ok_or_else(|| {
            GraphCompilerError::UnexpectedError(format!(
                "Abstract schedule assigned a node that doesn't exist: {:?}",
                scheduled_node
            ))
        })?;

    let plugin_id = plugin_host.id();
    let port_ids = plugin_host.port_ids();
    let shared_processor = plugin_host.shared_processor();
    let maybe_audio_ports_ext = plugin_host.audio_ports_ext();
    let maybe_note_ports_ext = plugin_host.note_ports_ext();

    // --- Construct a map that maps the PortChannelID of each port to its assigned buffer ------

    let mut assigned_audio_buffers: FnvHashMap<PortChannelID, (SharedBuffer<f32>, bool)> =
        FnvHashMap::default();
    let mut assigned_note_buffers: FnvHashMap<PortChannelID, (SharedBuffer<NoteIoEvent>, bool)> =
        FnvHashMap::default();
    let mut assigned_automation_in_buffer: Option<(SharedBuffer<AutomationIoEvent>, bool)> = None;
    let mut assigned_automation_out_buffer: Option<SharedBuffer<AutomationIoEvent>> = None;

    for assigned_buffer in
        scheduled_node.input_buffers.iter().chain(scheduled_node.output_buffers.iter())
    {
        let channel_id =
            port_ids.port_id_to_channel_id.get(&assigned_buffer.port_id).ok_or_else(|| {
                GraphCompilerError::UnexpectedError(format!(
                    "Abstract schedule assigned a buffer for port that doesn't exist {:?}",
                    scheduled_node
                ))
            })?;

        if assigned_buffer.type_index != channel_id.port_type.as_type_idx() {
            return Err(GraphCompilerError::UnexpectedError(format!(
                "Abstract schedule assigned the wrong type of buffer for port {:?}",
                scheduled_node
            )));
        }

        match channel_id.port_type {
            PortType::Audio => {
                let buffer = shared_pool
                    .buffers
                    .audio_buffer_pool
                    .initialized_buffer_at_index(assigned_buffer.buffer_index.0);

                if assigned_audio_buffers
                    .insert(*channel_id, (buffer, assigned_buffer.should_clear))
                    .is_some()
                {
                    return Err(GraphCompilerError::UnexpectedError(format!(
                        "Abstract schedule assigned multiple buffers to the same port {:?}",
                        scheduled_node
                    )));
                }
            }
            PortType::Note => {
                let buffer = shared_pool
                    .buffers
                    .note_buffer_pool
                    .buffer_at_index(assigned_buffer.buffer_index.0);

                if assigned_note_buffers
                    .insert(*channel_id, (buffer, assigned_buffer.should_clear))
                    .is_some()
                {
                    return Err(GraphCompilerError::UnexpectedError(format!(
                        "Abstract schedule assigned multiple buffers to the same port {:?}",
                        scheduled_node
                    )));
                }
            }
            PortType::Automation => {
                let buffer = shared_pool
                    .buffers
                    .automation_buffer_pool
                    .buffer_at_index(assigned_buffer.buffer_index.0);

                if channel_id.is_input {
                    if assigned_automation_in_buffer.is_some() {
                        return Err(GraphCompilerError::UnexpectedError(format!(
                            "Abstract schedule assigned multiple buffers to the param automation in port {:?}",
                            scheduled_node
                        )));
                    }
                    assigned_automation_in_buffer = Some((buffer, assigned_buffer.should_clear));
                } else {
                    if assigned_automation_out_buffer.is_some() {
                        return Err(GraphCompilerError::UnexpectedError(format!(
                            "Abstract schedule assigned multiple buffers to the param automation out port {:?}",
                            scheduled_node
                        )));
                    }
                    assigned_automation_out_buffer = Some(buffer);
                }
            }
        }
    }

    // --- Construct the final task using the constructed map from above --------------------

    if plugin_host.is_loaded() {
        loaded_plugin_task::construct_loaded_plugin_task(
            scheduled_node,
            shared_pool,
            plugin_id,
            shared_processor,
            maybe_audio_ports_ext.as_ref().unwrap(),
            maybe_note_ports_ext.as_ref().unwrap(),
            assigned_audio_buffers,
            assigned_note_buffers,
            assigned_automation_in_buffer,
            assigned_automation_out_buffer,
        )
    } else {
        // Plugin is unloaded
        unloaded_plugin_task::construct_unloaded_plugin_task(
            scheduled_node,
            maybe_audio_ports_ext,
            maybe_note_ports_ext,
            assigned_audio_buffers,
            assigned_note_buffers,
            assigned_automation_out_buffer,
        )
    }
}
