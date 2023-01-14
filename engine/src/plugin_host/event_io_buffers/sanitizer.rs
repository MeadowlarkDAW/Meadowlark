use fnv::FnvHashMap;
use meadowlark_plugin_api::ParamID;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use super::{AutomationIoEvent, AutomationIoEventType, NoteIoEvent, PluginIoEvent};

/// Sanitizes a plugin's event output stream, by wrapping an event iterator.
///
/// This means de-duplicating BeginGesture / EndGesture events, as well as
/// discarding any events where `header.time` is greater than or equal to
/// the number of frames in the current process cycle.
pub struct PluginEventOutputSanitizer {
    is_adjusting_parameter: FnvHashMap<ParamID, bool>,
}

impl PluginEventOutputSanitizer {
    pub fn new(param_capacity: usize) -> Self {
        let mut is_adjusting_parameter = FnvHashMap::default();
        is_adjusting_parameter.reserve(param_capacity * 2);

        Self { is_adjusting_parameter }
    }

    #[allow(unused)]
    pub fn reset(&mut self) {
        self.is_adjusting_parameter.clear()
    }

    #[inline]
    pub fn sanitize<I>(&mut self, iterator: I, frames: Option<u32>) -> ParamOutputSanitizerIter<I>
    where
        I: Iterator<Item = PluginIoEvent>,
    {
        ParamOutputSanitizerIter { sanitizer: self, iterator, frames }
    }
}

pub struct ParamOutputSanitizerIter<'a, I> {
    sanitizer: &'a mut PluginEventOutputSanitizer,
    iterator: I,
    frames: Option<u32>,
}

impl<'a, I> Iterator for ParamOutputSanitizerIter<'a, I>
where
    I: Iterator<Item = PluginIoEvent>,
{
    type Item = PluginIoEvent;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for event in self.iterator.by_ref() {
            match &event {
                PluginIoEvent::NoteEvent { event: NoteIoEvent { header, .. }, .. } => {
                    if let Some(frames) = self.frames {
                        if header.time >= frames {
                            log::warn!("Plugin outputted an event where `clap_event_header.time >= clap_process.frames_count`. Event was discarded.");
                            continue;
                        }
                    }

                    return Some(event);
                }
                PluginIoEvent::AutomationEvent {
                    event: AutomationIoEvent { parameter_id, event_type, header, .. },
                    ..
                } => {
                    if let Some(frames) = self.frames {
                        if header.time >= frames {
                            log::warn!("Plugin outputted an event where `clap_event_header.time >= clap_process.frames_count`. Event was discarded.");
                            continue;
                        }
                    }

                    let is_beginning = match event_type {
                        AutomationIoEventType::BeginGesture => true,
                        AutomationIoEventType::EndGesture => false,
                        _ => return Some(event),
                    };

                    match self.sanitizer.is_adjusting_parameter.entry(ParamID(*parameter_id)) {
                        Occupied(mut o) => {
                            if *o.get() == is_beginning {
                                log::warn!("Plugin outputted the event `CLAP_EVENT_PARAM_GESTURE_BEGIN` multiple times for the same parameter. Event was discarded.");
                                continue;
                            }
                            o.insert(is_beginning);
                        }
                        Vacant(v) => {
                            v.insert(is_beginning);
                        }
                    };

                    return Some(event);
                }
            }
        }

        None
    }
}
