use audio_graph::ScheduledNode;
use fnv::FnvHashMap;
use meadowlark_plugin_api::automation::AutomationIoEvent;
use meadowlark_plugin_api::buffer::SharedBuffer;
use meadowlark_plugin_api::ext::audio_ports::{MainPortsLayout, PluginAudioPortsExt};
use meadowlark_plugin_api::ext::note_ports::PluginNotePortsExt;
use smallvec::SmallVec;

use crate::plugin_host::event_io_buffers::NoteIoEvent;
use crate::processor_schedule::tasks::{Task, UnloadedPluginTask};

use super::super::super::error::GraphCompilerError;
use super::super::super::{PortChannelID, PortType};

/// In this task, audio and note data is passed through the main ports (if the plugin
/// has main in/out ports), and then all the other output buffers are cleared.
pub(super) fn construct_unloaded_plugin_task(
    scheduled_node: &ScheduledNode,
    maybe_audio_ports_ext: Option<&PluginAudioPortsExt>,
    maybe_note_ports_ext: Option<&PluginNotePortsExt>,
    mut assigned_audio_buffers: FnvHashMap<PortChannelID, (SharedBuffer<f32>, bool)>,
    mut assigned_note_buffers: FnvHashMap<PortChannelID, (SharedBuffer<NoteIoEvent>, bool)>,
    assigned_automation_out_buffer: Option<SharedBuffer<AutomationIoEvent>>,
) -> Result<Task, GraphCompilerError> {
    let mut audio_through: SmallVec<[(SharedBuffer<f32>, SharedBuffer<f32>); 4]> = SmallVec::new();
    let mut note_through: Option<(SharedBuffer<NoteIoEvent>, SharedBuffer<NoteIoEvent>)> = None;
    let mut clear_audio_out: SmallVec<[SharedBuffer<f32>; 4]> = SmallVec::new();
    let mut clear_note_out: SmallVec<[SharedBuffer<NoteIoEvent>; 2]> = SmallVec::new();
    let mut clear_automation_out: Option<SharedBuffer<AutomationIoEvent>> = None;

    if let Some(audio_ports_ext) = maybe_audio_ports_ext {
        if let MainPortsLayout::InOut = audio_ports_ext.main_ports_layout {
            let n_main_channels =
                audio_ports_ext.inputs[0].channels.min(audio_ports_ext.outputs[0].channels);

            for i in 0..n_main_channels {
                let in_channel_id = PortChannelID {
                    stable_id: audio_ports_ext.inputs[0].stable_id,
                    port_type: PortType::Audio,
                    is_input: true,
                    channel: i,
                };

                let out_channel_id = PortChannelID {
                    stable_id: audio_ports_ext.outputs[0].stable_id,
                    port_type: PortType::Audio,
                    is_input: false,
                    channel: i,
                };

                let in_buf = assigned_audio_buffers
                    .get(&in_channel_id)
                    .ok_or_else(|| {
                        GraphCompilerError::UnexpectedError(format!(
                            "Abstract schedule did not assign a buffer to every port in node {:?}",
                            scheduled_node
                        ))
                    })?
                    .0
                    .clone();
                let out_buf = assigned_audio_buffers
                    .remove(&out_channel_id)
                    .ok_or_else(|| {
                        GraphCompilerError::UnexpectedError(format!(
                            "Abstract schedule did not assign a buffer to every port in node {:?}",
                            scheduled_node
                        ))
                    })?
                    .0;

                audio_through.push((in_buf, out_buf));
            }
        }
    }

    if let Some(note_ports_ext) = maybe_note_ports_ext {
        if !note_ports_ext.inputs.is_empty() && !note_ports_ext.outputs.is_empty() {
            let in_channel_id = PortChannelID {
                stable_id: note_ports_ext.inputs[0].stable_id,
                port_type: PortType::Note,
                is_input: true,
                channel: 0,
            };

            let out_channel_id = PortChannelID {
                stable_id: note_ports_ext.outputs[0].stable_id,
                port_type: PortType::Note,
                is_input: false,
                channel: 0,
            };

            let in_buf = assigned_note_buffers
                .get(&in_channel_id)
                .ok_or_else(|| {
                    GraphCompilerError::UnexpectedError(format!(
                        "Abstract schedule did not assign a buffer to every port in node {:?}",
                        scheduled_node
                    ))
                })?
                .0
                .clone();
            let out_buf = assigned_note_buffers
                .remove(&out_channel_id)
                .ok_or_else(|| {
                    GraphCompilerError::UnexpectedError(format!(
                        "Abstract schedule did not assign a buffer to every port in node {:?}",
                        scheduled_node
                    ))
                })?
                .0;

            note_through = Some((in_buf, out_buf));
        }
    }

    for (channel_id, (buffer, _)) in assigned_audio_buffers.iter() {
        if !channel_id.is_input {
            clear_audio_out.push(buffer.clone());
        }
    }
    for (channel_id, (buffer, _)) in assigned_note_buffers.iter() {
        if !channel_id.is_input {
            clear_note_out.push(buffer.clone());
        }
    }
    if let Some(buffer) = assigned_automation_out_buffer {
        clear_automation_out = Some(buffer);
    }

    Ok(Task::UnloadedPlugin(UnloadedPluginTask {
        audio_through,
        note_through,
        clear_audio_out,
        clear_note_out,
        clear_automation_out,
    }))
}
