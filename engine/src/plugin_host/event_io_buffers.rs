use clack_host::events::event_types::NoteEvent as ClackNoteEvent;
use clack_host::events::event_types::*;
use clack_host::events::io::EventBuffer;
use clack_host::events::spaces::CoreEventSpace;
use clack_host::events::{Event, EventHeader as ClackEventHeader, UnknownEvent};
use smallvec::SmallVec;

use meadowlark_plugin_api::automation::{AutomationIoEvent, AutomationIoEventType, IoEventHeader};
use meadowlark_plugin_api::buffer::SharedBuffer;
use meadowlark_plugin_api::ParamID;

mod sanitizer;

pub(crate) use sanitizer::PluginEventOutputSanitizer;

use crate::utils::reducing_queue::ReducFnvProducerRefMut;

use super::channel::ProcToMainParamValue;

// TODO: remove pubs
pub(crate) struct PluginEventIoBuffers {
    pub note_in_buffers: SmallVec<[SharedBuffer<NoteIoEvent>; 2]>,
    pub note_out_buffers: SmallVec<[SharedBuffer<NoteIoEvent>; 2]>,

    pub clear_note_in_buffers: SmallVec<[SharedBuffer<NoteIoEvent>; 2]>,

    pub automation_in_buffer: Option<(SharedBuffer<AutomationIoEvent>, bool)>,
    /// Only for internal plugin (e.g. timeline or macros)
    pub automation_out_buffer: Option<SharedBuffer<AutomationIoEvent>>,

    pub main_note_through_when_bypassed: bool,
}

impl PluginEventIoBuffers {
    pub fn clear_before_process(&mut self) {
        if let Some((buffer, do_clear)) = &self.automation_in_buffer {
            if *do_clear {
                buffer.truncate();
            }
        }
        if let Some(buffer) = &mut self.automation_out_buffer {
            buffer.truncate();
        }

        for buffer in self.clear_note_in_buffers.iter().chain(self.note_out_buffers.iter()) {
            buffer.truncate();
        }
    }

    pub fn write_input_events(
        &self,
        raw_event_buffer: &mut EventBuffer,
        plugin_instance_id: u64,
    ) -> (bool, bool) {
        let wrote_note_event = self.write_input_note_events(raw_event_buffer);
        let wrote_param_event =
            self.write_input_automation_events(raw_event_buffer, plugin_instance_id);

        (wrote_note_event, wrote_param_event)
    }

    fn write_input_note_events(&self, raw_event_buffer: &mut EventBuffer) -> bool {
        let mut wrote_note_event = false;

        for (note_port_index, buffer) in self.note_in_buffers.iter().enumerate() {
            for event in buffer.borrow().data.iter() {
                let event = PluginIoEvent::NoteEvent {
                    note_port_index: note_port_index as i16,
                    event: *event,
                };
                event.write_to_clap_buffer(raw_event_buffer);
                wrote_note_event = true;
            }
        }

        wrote_note_event
    }

    fn write_input_automation_events(
        &self,
        raw_event_buffer: &mut EventBuffer,
        plugin_instance_id: u64,
    ) -> bool {
        let mut wrote_event = false;

        if let Some((in_buf, _)) = &self.automation_in_buffer {
            for event in in_buf.borrow().data.iter() {
                if event.plugin_instance_id == plugin_instance_id {
                    let event = PluginIoEvent::AutomationEvent { event: *event };
                    event.write_to_clap_buffer(raw_event_buffer);
                    wrote_event = true;
                }
            }
        }

        wrote_event
    }

    pub fn read_output_events(
        &mut self,
        raw_event_buffer: &EventBuffer,
        mut external_parameter_queue: Option<
            &mut ReducFnvProducerRefMut<ParamID, ProcToMainParamValue>,
        >,
        sanitizer: &mut PluginEventOutputSanitizer,
        frames: u32,
    ) {
        let events_iter = raw_event_buffer.iter().filter_map(PluginIoEvent::read_from_clap);
        let events_iter = sanitizer.sanitize(events_iter, Some(frames));

        for event in events_iter {
            match event {
                PluginIoEvent::NoteEvent { note_port_index, event } => {
                    if let Some(b) = self.note_out_buffers.get(note_port_index as usize) {
                        b.borrow_mut().data.push(event)
                    }
                }
                PluginIoEvent::AutomationEvent { event } => {
                    if let Some(queue) = external_parameter_queue.as_mut() {
                        if let Some(value) =
                            ProcToMainParamValue::from_param_event(event.event_type)
                        {
                            queue.set_or_update(ParamID::new(event.parameter_id), value);
                        }
                    }
                }
            }
        }
    }

    pub fn bypassed(&mut self) {
        for note_out in self.note_out_buffers.iter() {
            note_out.truncate();
        }

        if let Some(automation_out) = &self.automation_out_buffer {
            automation_out.truncate();
        }

        // TODO: More note through ports when bypassed?
        if self.main_note_through_when_bypassed {
            let in_buf = self.note_in_buffers[0].borrow();
            let mut out_buf = self.note_out_buffers[0].borrow_mut();

            for event in in_buf.data.iter() {
                out_buf.data.push(*event);
            }
        }
    }
}

// Contents of NoteBuffer
#[derive(Copy, Clone)]
pub struct NoteIoEvent {
    pub header: IoEventHeader,
    pub channel: i16,
    pub key: i16,
    pub event_type: NoteIoEventType,
}

#[derive(Copy, Clone)]
pub enum NoteIoEventType {
    On { velocity: f64 },
    Expression { expression_type: NoteExpressionType, value: f64 },
    Choke,
    Off { velocity: f64 },
}

#[derive(Copy, Clone)]
pub enum PluginIoEvent {
    NoteEvent { note_port_index: i16, event: NoteIoEvent },
    AutomationEvent { event: AutomationIoEvent },
}

impl PluginIoEvent {
    pub fn read_from_clap(clap_event: &UnknownEvent) -> Option<Self> {
        match clap_event.as_core_event()? {
            CoreEventSpace::NoteOn(NoteOnEvent(e)) => Some(PluginIoEvent::NoteEvent {
                note_port_index: e.port_index(),
                event: NoteIoEvent {
                    channel: e.channel(),
                    key: e.key(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: NoteIoEventType::On { velocity: e.velocity() },
                },
            }),
            CoreEventSpace::NoteOff(NoteOffEvent(e)) => Some(PluginIoEvent::NoteEvent {
                note_port_index: e.port_index(),
                event: NoteIoEvent {
                    channel: e.channel(),
                    key: e.key(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: NoteIoEventType::Off { velocity: e.velocity() },
                },
            }),
            CoreEventSpace::NoteChoke(NoteChokeEvent(e)) => Some(PluginIoEvent::NoteEvent {
                note_port_index: e.port_index(),
                event: NoteIoEvent {
                    channel: e.channel(),
                    key: e.key(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: NoteIoEventType::Choke,
                },
            }),
            CoreEventSpace::NoteExpression(e) => Some(PluginIoEvent::NoteEvent {
                note_port_index: e.port_index(),
                event: NoteIoEvent {
                    channel: e.channel(),
                    key: e.key(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: NoteIoEventType::Expression {
                        expression_type: e.expression_type()?,
                        value: e.value(),
                    },
                },
            }),

            CoreEventSpace::ParamValue(e) => Some(PluginIoEvent::AutomationEvent {
                event: AutomationIoEvent {
                    plugin_instance_id: 0,
                    parameter_id: e.param_id(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: AutomationIoEventType::Value(e.value()),
                    cookie: Some(e.cookie()),
                },
            }),
            CoreEventSpace::ParamMod(e) => Some(PluginIoEvent::AutomationEvent {
                event: AutomationIoEvent {
                    plugin_instance_id: 0,
                    parameter_id: e.param_id(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: AutomationIoEventType::Modulation(e.value()),
                    cookie: Some(e.cookie()),
                },
            }),
            CoreEventSpace::ParamGestureBegin(e) => Some(PluginIoEvent::AutomationEvent {
                event: AutomationIoEvent {
                    plugin_instance_id: 0,
                    parameter_id: e.param_id(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: AutomationIoEventType::BeginGesture,
                    cookie: None,
                },
            }),
            CoreEventSpace::ParamGestureEnd(e) => Some(PluginIoEvent::AutomationEvent {
                event: AutomationIoEvent {
                    plugin_instance_id: 0,
                    parameter_id: e.param_id(),
                    header: IoEventHeader { time: e.header().time() },
                    event_type: AutomationIoEventType::EndGesture,
                    cookie: None,
                },
            }),

            // TODO: handle MIDI events & note end events
            CoreEventSpace::Transport(_) => {
                log::warn!("Plugin outputted a `CLAP_EVENT_TRANSPORT` event. Event was discarded.");
                None
            }

            _ => None,
        }
    }

    pub fn write_to_clap_buffer(&self, buffer: &mut EventBuffer) {
        match self {
            PluginIoEvent::NoteEvent {
                note_port_index,
                event: NoteIoEvent { event_type, key, channel, header: IoEventHeader { time } },
            } => match event_type {
                NoteIoEventType::On { velocity } => buffer.push(
                    NoteOnEvent(ClackNoteEvent::new(
                        ClackEventHeader::new(*time),
                        -1,
                        *note_port_index,
                        *key,
                        *channel,
                        *velocity,
                    ))
                    .as_unknown(),
                ),
                NoteIoEventType::Expression { expression_type, value } => buffer.push(
                    NoteExpressionEvent::new(
                        ClackEventHeader::new(*time),
                        -1,
                        *note_port_index,
                        *key,
                        *channel,
                        *value,
                        *expression_type,
                    )
                    .as_unknown(),
                ),

                NoteIoEventType::Choke => buffer.push(
                    NoteChokeEvent(ClackNoteEvent::new(
                        ClackEventHeader::new(*time),
                        -1,
                        *note_port_index,
                        *key,
                        *channel,
                        0.0,
                    ))
                    .as_unknown(),
                ),

                NoteIoEventType::Off { velocity } => buffer.push(
                    NoteOffEvent(ClackNoteEvent::new(
                        ClackEventHeader::new(*time),
                        -1,
                        *note_port_index,
                        *key,
                        *channel,
                        *velocity,
                    ))
                    .as_unknown(),
                ),
            },
            PluginIoEvent::AutomationEvent {
                event:
                    AutomationIoEvent {
                        header: IoEventHeader { time },
                        parameter_id,
                        event_type,
                        plugin_instance_id: _,
                        cookie,
                    },
            } => {
                match event_type {
                    AutomationIoEventType::Value(value) => {
                        if let Some(cookie) = cookie {
                            buffer.push(
                                ParamValueEvent::new(
                                    ClackEventHeader::new(*time),
                                    *cookie,
                                    -1,
                                    *parameter_id,
                                    -1,
                                    -1,
                                    -1,
                                    *value,
                                )
                                .as_unknown(),
                            )
                        } else {
                            log::error!("Could not write automation event to CLAP buffer: event had no cookie");
                        }
                    }
                    AutomationIoEventType::Modulation(modulation_amount) => {
                        if let Some(cookie) = cookie {
                            buffer.push(
                                ParamModEvent::new(
                                    ClackEventHeader::new(*time),
                                    *cookie,
                                    -1,
                                    *parameter_id,
                                    -1,
                                    -1,
                                    -1,
                                    *modulation_amount,
                                )
                                .as_unknown(),
                            )
                        } else {
                            log::error!("Could not write automation event to CLAP buffer: event had no cookie");
                        }
                    }
                    AutomationIoEventType::BeginGesture => buffer.push(
                        ParamGestureBeginEvent::new(ClackEventHeader::new(*time), *parameter_id)
                            .as_unknown(),
                    ),
                    AutomationIoEventType::EndGesture => buffer.push(
                        ParamGestureEndEvent::new(ClackEventHeader::new(*time), *parameter_id)
                            .as_unknown(),
                    ),
                }
            }
        }
    }
}
