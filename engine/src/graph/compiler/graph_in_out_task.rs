use audio_graph::ScheduledNode;
use meadowlark_plugin_api::buffer::SharedBuffer;
use smallvec::{smallvec, SmallVec};

use crate::processor_schedule::tasks::{GraphInTask, GraphOutTask};

use super::super::error::GraphCompilerError;
use super::super::shared_pools::GraphSharedPools;
use super::super::PortType;

pub(super) fn construct_graph_in_task(
    scheduled_node: &ScheduledNode,
    shared_pool: &mut GraphSharedPools,
    num_graph_in_audio_ports: usize,
) -> Result<GraphInTask, GraphCompilerError> {
    // --- Construct a map that maps the index (channel) of each port to its assigned buffer

    let mut audio_out_slots: SmallVec<[Option<SharedBuffer<f32>>; 8]> =
        smallvec![None; num_graph_in_audio_ports];
    for output_buffer in scheduled_node.output_buffers.iter() {
        match output_buffer.type_index {
            PortType::AUDIO_TYPE_IDX => {
                let buffer = shared_pool
                    .buffers
                    .audio_buffer_pool
                    .initialized_buffer_at_index(output_buffer.buffer_index.0);

                let buffer_slot =
                    audio_out_slots.get_mut(output_buffer.port_id.0 as usize).ok_or_else(|| {
                        GraphCompilerError::UnexpectedError(format!(
                    "Abstract schedule assigned buffer to graph in node with invalid port id {:?}",
                    output_buffer
                ))
                    })?;

                *buffer_slot = Some(buffer);
            }
            PortType::NOTE_TYPE_IDX => {
                // TODO: Note buffers in graph input.
            }
            _ => {
                return Err(GraphCompilerError::UnexpectedError(format!(
                    "Abstract schedule assigned buffer with invalid type index on graph in node {:?}",
                    output_buffer
                )));
            }
        }
    }

    // --- Construct the final task using the constructed map from above --------------------

    let mut audio_in: SmallVec<[SharedBuffer<f32>; 8]> =
        SmallVec::with_capacity(num_graph_in_audio_ports);
    for buffer_slot in audio_out_slots.drain(..) {
        let buffer = buffer_slot.ok_or_else(|| {
            GraphCompilerError::UnexpectedError(format!(
                "Abstract schedule did not assign a buffer to all ports on graph in node {:?}",
                scheduled_node
            ))
        })?;

        audio_in.push(buffer);
    }

    Ok(GraphInTask { audio_in })
}

pub(super) fn construct_graph_out_task(
    scheduled_node: &ScheduledNode,
    shared_pool: &mut GraphSharedPools,
    num_graph_out_audio_ports: usize,
) -> Result<GraphOutTask, GraphCompilerError> {
    // --- Construct a map that maps the index (channel) of each port to its assigned buffer

    let mut audio_in_slots: SmallVec<[Option<SharedBuffer<f32>>; 8]> =
        smallvec![None; num_graph_out_audio_ports];
    for input_buffer in scheduled_node.input_buffers.iter() {
        match input_buffer.type_index {
            PortType::AUDIO_TYPE_IDX => {
                let buffer = shared_pool
                    .buffers
                    .audio_buffer_pool
                    .initialized_buffer_at_index(input_buffer.buffer_index.0);

                let buffer_slot =
                    audio_in_slots.get_mut(input_buffer.port_id.0 as usize).ok_or_else(|| {
                        GraphCompilerError::UnexpectedError(format!(
                    "Abstract schedule assigned buffer to graph out node with invalid port id {:?}",
                    input_buffer
                ))
                    })?;

                *buffer_slot = Some(buffer);
            }
            PortType::NOTE_TYPE_IDX => {
                // TODO: Note buffers in graph input.
            }
            _ => {
                return Err(GraphCompilerError::UnexpectedError(format!(
                    "Abstract schedule assigned buffer with invalid type index on graph out node {:?}",
                    input_buffer
                )));
            }
        }
    }

    // --- Construct the final task using the constructed map from above --------------------

    let mut audio_out: SmallVec<[SharedBuffer<f32>; 8]> =
        SmallVec::with_capacity(num_graph_out_audio_ports);
    for buffer_slot in audio_in_slots.drain(..) {
        let buffer = buffer_slot.ok_or_else(|| {
            GraphCompilerError::UnexpectedError(format!(
                "Abstract schedule did not assign a buffer to all ports on graph out node {:?}",
                scheduled_node
            ))
        })?;

        audio_out.push(buffer);
    }

    Ok(GraphOutTask { audio_out })
}
