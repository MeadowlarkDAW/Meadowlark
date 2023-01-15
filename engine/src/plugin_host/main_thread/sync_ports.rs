use audio_graph::{AudioGraphHelper, EdgeID, PortID};
use fnv::{FnvHashMap, FnvHashSet};
use meadowlark_plugin_api::ext::audio_ports::{MainPortsLayout, PluginAudioPortsExt};
use meadowlark_plugin_api::ext::note_ports::PluginNotePortsExt;

use crate::graph::{EngineEdgeID, PortChannelID, PortType};

use super::super::error::ActivatePluginError;
use super::PluginHostMainThread;

pub(super) fn sync_latency_in_graph(
    plugin_host: &mut PluginHostMainThread,
    graph_helper: &mut AudioGraphHelper,
    new_latency: i64,
) {
    // Update the latency on the node assigned to this plugin.
    graph_helper.set_node_latency(plugin_host.id._node_id().into(), new_latency as f64).unwrap();
}

/// Adds/removes ports from the abstract graph according to the plugin's new
/// audio ports and note ports extensions. Also updates the new latency for
/// the node in the abstract graph.
///
/// On success, returns:
/// - a list of all edges that were removed as a result of the plugin
/// removing some of its ports
/// - `true` if the audio graph needs to be recompiled as a result of the
/// plugin adding/removing ports.
pub(super) fn sync_ports_in_graph(
    plugin_host: &mut PluginHostMainThread,
    graph_helper: &mut AudioGraphHelper,
    edge_id_to_ds_edge_id: &mut FnvHashMap<EdgeID, EngineEdgeID>,
    new_audio_ports: &Option<PluginAudioPortsExt>,
    new_note_ports: &Option<PluginNotePortsExt>,
    coll_handle: &basedrop::Handle,
) -> Result<(Vec<EngineEdgeID>, bool), ActivatePluginError> {
    // ----------------------------------------------------------------------------

    // Make sure that the plugin did not assign the same ID to multiple
    // input or output ports.
    let mut id_alias_check: FnvHashSet<u32> = FnvHashSet::default();
    let mut status = Ok(());

    if let Some(audio_ports) = new_audio_ports {
        for audio_in_port in audio_ports.inputs.iter() {
            if !id_alias_check.insert(audio_in_port.stable_id) {
                status = Err(ActivatePluginError::AudioPortsExtDuplicateID {
                    is_input: true,
                    id: audio_in_port.stable_id,
                });
            }
        }
        id_alias_check.clear();
        for audio_out_port in audio_ports.outputs.iter() {
            if !id_alias_check.insert(audio_out_port.stable_id) {
                status = Err(ActivatePluginError::AudioPortsExtDuplicateID {
                    is_input: false,
                    id: audio_out_port.stable_id,
                });
            }
        }
    }
    if let Err(e) = status {
        plugin_host.schedule_deactivate(coll_handle);
        return Err(e);
    }
    id_alias_check.clear();
    if let Some(note_ports) = new_note_ports {
        for note_in_port in note_ports.inputs.iter() {
            if !id_alias_check.insert(note_in_port.stable_id) {
                status = Err(ActivatePluginError::NotePortsExtDuplicateID {
                    is_input: true,
                    id: note_in_port.stable_id,
                });
            }
        }
        id_alias_check.clear();
        for note_out_port in note_ports.outputs.iter() {
            if !id_alias_check.insert(note_out_port.stable_id) {
                status = Err(ActivatePluginError::NotePortsExtDuplicateID {
                    is_input: false,
                    id: note_out_port.stable_id,
                });
            }
        }
    }
    if let Err(e) = status {
        plugin_host.schedule_deactivate(coll_handle);
        return Err(e);
    }

    // ----------------------------------------------------------------------------

    // Clone the list of previous port channels so we know later which
    // ports the plugin removed.
    let mut prev_channel_ids = plugin_host.port_ids.channel_id_to_port_id.clone();

    plugin_host.port_ids.channel_id_to_port_id.clear();
    plugin_host.port_ids.port_id_to_channel_id.clear();
    plugin_host.port_ids.automation_in_port_id = None;
    plugin_host.port_ids.automation_out_port_id = None;

    let mut needs_recompile = false;

    // ---  Audio Ports  ----------------------------------------------------------
    if let Some(new_audio_ports) = new_audio_ports {
        plugin_host.port_ids.main_audio_in_port_ids.clear();
        plugin_host.port_ids.main_audio_out_port_ids.clear();
        plugin_host.port_ids.main_note_in_port_id = None;
        plugin_host.port_ids.main_note_out_port_id = None;

        // Sync audio input ports.
        for (audio_port_i, audio_in_port) in new_audio_ports.inputs.iter().enumerate() {
            for channel_i in 0..audio_in_port.channels {
                let channel_id = PortChannelID {
                    stable_id: audio_in_port.stable_id,
                    port_type: PortType::Audio,
                    is_input: true,
                    channel: channel_i,
                };

                let port_id = if let Some(port_id) = prev_channel_ids.remove(&channel_id) {
                    port_id
                } else {
                    needs_recompile = true;

                    let new_port_id = plugin_host.free_port_ids.pop().unwrap_or_else(|| {
                        plugin_host.next_port_id += 1;
                        PortID(plugin_host.next_port_id - 1)
                    });

                    graph_helper
                        .add_port(
                            plugin_host.id._node_id().into(),
                            new_port_id,
                            PortType::Audio.as_type_idx(),
                            true,
                        )
                        .unwrap();

                    new_port_id
                };

                plugin_host.port_ids.channel_id_to_port_id.insert(channel_id, port_id);
                plugin_host.port_ids.port_id_to_channel_id.insert(port_id, channel_id);

                if audio_port_i == 0 {
                    match new_audio_ports.main_ports_layout {
                        MainPortsLayout::InOut | MainPortsLayout::InOnly => {
                            plugin_host.port_ids.main_audio_in_port_ids.push(port_id);
                        }
                        _ => {}
                    }
                }
            }
        }

        // Sync audio output ports.
        for (audio_port_i, audio_out_port) in new_audio_ports.outputs.iter().enumerate() {
            for channel_i in 0..audio_out_port.channels {
                let channel_id = PortChannelID {
                    stable_id: audio_out_port.stable_id,
                    port_type: PortType::Audio,
                    is_input: false,
                    channel: channel_i,
                };

                let port_id = if let Some(port_id) = prev_channel_ids.remove(&channel_id) {
                    port_id
                } else {
                    needs_recompile = true;

                    let new_port_id = plugin_host.free_port_ids.pop().unwrap_or_else(|| {
                        plugin_host.next_port_id += 1;
                        PortID(plugin_host.next_port_id - 1)
                    });

                    graph_helper
                        .add_port(
                            plugin_host.id._node_id().into(),
                            new_port_id,
                            PortType::Audio.as_type_idx(),
                            false,
                        )
                        .unwrap();

                    new_port_id
                };

                plugin_host.port_ids.channel_id_to_port_id.insert(channel_id, port_id);
                plugin_host.port_ids.port_id_to_channel_id.insert(port_id, channel_id);

                if audio_port_i == 0 {
                    match new_audio_ports.main_ports_layout {
                        MainPortsLayout::InOut | MainPortsLayout::OutOnly => {
                            plugin_host.port_ids.main_audio_out_port_ids.push(port_id);
                        }
                        _ => {}
                    }
                }
            }
        }
    } else {
        for (channel_id, port_id) in
            prev_channel_ids.iter().filter(|(c, _)| c.port_type == PortType::Audio)
        {
            plugin_host.port_ids.channel_id_to_port_id.insert(*channel_id, *port_id);
            plugin_host.port_ids.port_id_to_channel_id.insert(*port_id, *channel_id);
        }
    }

    // ---  Automation Ports  -----------------------------------------------------

    const IN_AUTOMATION_CHANNEL_ID: PortChannelID =
        PortChannelID { port_type: PortType::Automation, stable_id: 0, is_input: true, channel: 0 };
    const OUT_AUTOMATION_CHANNEL_ID: PortChannelID = PortChannelID {
        port_type: PortType::Automation,
        stable_id: 0,
        is_input: false,
        channel: 0,
    };

    // Plugins always have one automation in port.
    let automation_in_port_id =
        if let Some(port_id) = prev_channel_ids.remove(&IN_AUTOMATION_CHANNEL_ID) {
            port_id
        } else {
            needs_recompile = true;

            let new_port_id = plugin_host.free_port_ids.pop().unwrap_or_else(|| {
                plugin_host.next_port_id += 1;
                PortID(plugin_host.next_port_id - 1)
            });

            graph_helper
                .add_port(
                    plugin_host.id._node_id().into(),
                    new_port_id,
                    PortType::Automation.as_type_idx(),
                    true,
                )
                .unwrap();

            new_port_id
        };
    plugin_host
        .port_ids
        .channel_id_to_port_id
        .insert(IN_AUTOMATION_CHANNEL_ID, automation_in_port_id);
    plugin_host
        .port_ids
        .port_id_to_channel_id
        .insert(automation_in_port_id, IN_AUTOMATION_CHANNEL_ID);
    plugin_host.port_ids.automation_in_port_id = Some(automation_in_port_id);

    if plugin_host.plug_main_thread.has_automation_out_port() {
        let automation_out_port_id =
            if let Some(port_id) = prev_channel_ids.remove(&OUT_AUTOMATION_CHANNEL_ID) {
                port_id
            } else {
                needs_recompile = true;

                let new_port_id = plugin_host.free_port_ids.pop().unwrap_or_else(|| {
                    plugin_host.next_port_id += 1;
                    PortID(plugin_host.next_port_id - 1)
                });

                graph_helper
                    .add_port(
                        plugin_host.id._node_id().into(),
                        new_port_id,
                        PortType::Automation.as_type_idx(),
                        false,
                    )
                    .unwrap();

                new_port_id
            };
        plugin_host
            .port_ids
            .channel_id_to_port_id
            .insert(OUT_AUTOMATION_CHANNEL_ID, automation_out_port_id);
        plugin_host
            .port_ids
            .port_id_to_channel_id
            .insert(automation_out_port_id, OUT_AUTOMATION_CHANNEL_ID);
        plugin_host.port_ids.automation_out_port_id = Some(automation_out_port_id);
    }

    // ---  Note Ports  -----------------------------------------------------------

    if let Some(new_note_ports) = new_note_ports {
        // Sync note in ports.
        for (i, note_in_port) in new_note_ports.inputs.iter().enumerate() {
            let channel_id = PortChannelID {
                port_type: PortType::Note,
                stable_id: note_in_port.stable_id,
                is_input: true,
                channel: 0,
            };

            let port_id = if let Some(port_id) = prev_channel_ids.remove(&channel_id) {
                port_id
            } else {
                needs_recompile = true;

                let new_port_id = plugin_host.free_port_ids.pop().unwrap_or_else(|| {
                    plugin_host.next_port_id += 1;
                    PortID(plugin_host.next_port_id - 1)
                });

                graph_helper
                    .add_port(
                        plugin_host.id._node_id().into(),
                        new_port_id,
                        PortType::Note.as_type_idx(),
                        true,
                    )
                    .unwrap();

                new_port_id
            };

            plugin_host.port_ids.channel_id_to_port_id.insert(channel_id, port_id);
            plugin_host.port_ids.port_id_to_channel_id.insert(port_id, channel_id);

            if i == 0 {
                plugin_host.port_ids.main_note_in_port_id = Some(port_id);
            }
        }

        // Sync note out ports
        for (i, note_out_port) in new_note_ports.outputs.iter().enumerate() {
            let channel_id = PortChannelID {
                port_type: PortType::Note,
                stable_id: note_out_port.stable_id,
                is_input: false,
                channel: 0,
            };

            let port_id = if let Some(port_id) = prev_channel_ids.remove(&channel_id) {
                port_id
            } else {
                needs_recompile = true;

                let new_port_id = plugin_host.free_port_ids.pop().unwrap_or_else(|| {
                    plugin_host.next_port_id += 1;
                    PortID(plugin_host.next_port_id - 1)
                });

                graph_helper
                    .add_port(
                        plugin_host.id._node_id().into(),
                        new_port_id,
                        PortType::Note.as_type_idx(),
                        false,
                    )
                    .unwrap();

                new_port_id
            };

            plugin_host.port_ids.channel_id_to_port_id.insert(channel_id, port_id);
            plugin_host.port_ids.port_id_to_channel_id.insert(port_id, channel_id);

            if i == 0 {
                plugin_host.port_ids.main_note_out_port_id = Some(port_id);
            }
        }
    } else {
        for (channel_id, port_id) in
            prev_channel_ids.iter().filter(|(c, _)| c.port_type == PortType::Note)
        {
            plugin_host.port_ids.channel_id_to_port_id.insert(*channel_id, *port_id);
            plugin_host.port_ids.port_id_to_channel_id.insert(*port_id, *channel_id);
        }
    }

    // ----------------------------------------------------------------------------

    // Remove all the ports that are no longer being used by the plugin.
    let mut removed_edges: Vec<EngineEdgeID> = Vec::new();
    for (channel_id, port_to_remove_id) in prev_channel_ids.drain() {
        if (channel_id.port_type == PortType::Audio && new_audio_ports.is_none())
            || (channel_id.port_type == PortType::Note && new_note_ports.is_none())
        {
            continue;
        }

        // We need to recompile the audio graph if the plugin has
        // removed any of its ports.
        needs_recompile = true;

        let removed_edges_res =
            graph_helper.remove_port(plugin_host.id._node_id().into(), port_to_remove_id).unwrap();

        for edge_id in removed_edges_res.iter() {
            if let Some(ds_edge_id) = edge_id_to_ds_edge_id.remove(edge_id) {
                removed_edges.push(ds_edge_id);
            } else {
                panic!("Helper disconnected an edge that doesn't exist in graph: {:?}", edge_id);
            }
        }

        plugin_host.free_port_ids.push(port_to_remove_id);
    }

    Ok((removed_edges, needs_recompile))
}
