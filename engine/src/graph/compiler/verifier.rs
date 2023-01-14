use fnv::FnvHashSet;
use meadowlark_plugin_api::buffer::{DebugBufferID, RawAudioChannelBuffers};

use crate::processor_schedule::{tasks::Task, ProcessorSchedule};

use super::super::error::VerifyScheduleError;

pub(crate) struct Verifier {
    plugin_instances: FnvHashSet<u64>,
    buffer_instances: FnvHashSet<DebugBufferID>,
}

impl Verifier {
    pub fn new() -> Self {
        let mut plugin_instances: FnvHashSet<u64> = FnvHashSet::default();
        let mut buffer_instances: FnvHashSet<DebugBufferID> = FnvHashSet::default();
        plugin_instances.reserve(1024);
        buffer_instances.reserve(1024);

        Verifier { plugin_instances, buffer_instances }
    }

    /// Verify that the schedule is sound (no race conditions).
    ///
    /// This is probably expensive, but I would like to keep this check here until we are very
    /// confident in the stability and soundness of this audio graph compiler.
    ///
    /// We are using reference-counted pointers (`basedrop::Shared`) for everything, so we shouldn't
    /// ever run into a situation where the schedule assigns a pointer to a buffer or a node that
    /// doesn't exist in memory.
    ///
    /// However, it is still very possible to have race condition bugs in the schedule, such as
    /// the same buffer being assigned multiple times within the same task, or the same buffer
    /// appearing multiple times between parallel tasks (once we have multithreaded scheduling).
    pub fn verify_schedule_for_race_conditions(
        &mut self,
        schedule: &ProcessorSchedule,
    ) -> Result<(), VerifyScheduleError> {
        // TODO: verifying that there are not data races between parallel threads once we
        // have multithreaded scheduling.

        self.plugin_instances.clear();

        for task in schedule.tasks().iter() {
            self.buffer_instances.clear();

            match task {
                Task::Plugin(t) => {
                    if !self.plugin_instances.insert(t.plugin_id.unique_id()) {
                        return Err(VerifyScheduleError::PluginInstanceAppearsTwiceInSchedule {
                            plugin_id: t.plugin_id.clone(),
                        });
                    }

                    for port_buffer in t.buffers.audio_in.iter() {
                        match &port_buffer._raw_channels {
                            RawAudioChannelBuffers::F32(buffers) => {
                                for b in buffers.iter() {
                                    if !self.buffer_instances.insert(b.id()) {
                                        return Err(
                                            VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                                buffer_id: b.id(),
                                                task_info: format!("{:?}", &task),
                                            },
                                        );
                                    }
                                }
                            }
                            RawAudioChannelBuffers::F64(buffers) => {
                                for b in buffers.iter() {
                                    if !self.buffer_instances.insert(b.id()) {
                                        return Err(
                                            VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                                buffer_id: b.id(),
                                                task_info: format!("{:?}", &task),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }

                    for port_buffer in t.buffers.audio_out.iter() {
                        match &port_buffer._raw_channels {
                            RawAudioChannelBuffers::F32(buffers) => {
                                for b in buffers.iter() {
                                    if !self.buffer_instances.insert(b.id()) {
                                        return Err(
                                            VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                                buffer_id: b.id(),
                                                task_info: format!("{:?}", &task),
                                            },
                                        );
                                    }
                                }
                            }
                            RawAudioChannelBuffers::F64(buffers) => {
                                for b in buffers.iter() {
                                    if !self.buffer_instances.insert(b.id()) {
                                        return Err(
                                            VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                                buffer_id: b.id(),
                                                task_info: format!("{:?}", &task),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Task::AudioSum(t) => {
                    // This could be made just a warning and not an error, but it's still not what
                    // we want to happen.
                    if t.audio_in.len() < 2 {
                        return Err(VerifyScheduleError::SumNodeWithLessThanTwoInputs {
                            num_inputs: t.audio_in.len(),
                            task_info: format!("{:?}", &task),
                        });
                    }

                    for b in t.audio_in.iter() {
                        if !self.buffer_instances.insert(b.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }
                    if !self.buffer_instances.insert(t.audio_out.id()) {
                        return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                            buffer_id: t.audio_out.id(),
                            task_info: format!("{:?}", &task),
                        });
                    }
                }
                Task::NoteSum(t) => {
                    // This could be made just a warning and not an error, but it's still not what
                    // we want to happen.
                    if t.note_in.len() < 2 {
                        return Err(VerifyScheduleError::SumNodeWithLessThanTwoInputs {
                            num_inputs: t.note_in.len(),
                            task_info: format!("{:?}", &task),
                        });
                    }

                    for b in t.note_in.iter() {
                        if !self.buffer_instances.insert(b.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }
                    if !self.buffer_instances.insert(t.note_out.id()) {
                        return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                            buffer_id: t.note_out.id(),
                            task_info: format!("{:?}", &task),
                        });
                    }
                }
                Task::AutomationSum(t) => {
                    // This could be made just a warning and not an error, but it's still not what
                    // we want to happen.
                    if t.input.len() < 2 {
                        return Err(VerifyScheduleError::SumNodeWithLessThanTwoInputs {
                            num_inputs: t.input.len(),
                            task_info: format!("{:?}", &task),
                        });
                    }

                    for b in t.input.iter() {
                        if !self.buffer_instances.insert(b.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }
                    if !self.buffer_instances.insert(t.output.id()) {
                        return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                            buffer_id: t.output.id(),
                            task_info: format!("{:?}", &task),
                        });
                    }
                }
                Task::AudioDelayComp(t) => {
                    if t.audio_in.id() == t.audio_out.id() {
                        return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                            buffer_id: t.audio_in.id(),
                            task_info: format!("{:?}", &task),
                        });
                    }
                }
                Task::NoteDelayComp(t) => {
                    if t.note_in.id() == t.note_out.id() {
                        return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                            buffer_id: t.note_in.id(),
                            task_info: format!("{:?}", &task),
                        });
                    }
                }
                Task::AutomationDelayComp(t) => {
                    if t.input.id() == t.output.id() {
                        return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                            buffer_id: t.input.id(),
                            task_info: format!("{:?}", &task),
                        });
                    }
                }
                Task::UnloadedPlugin(t) => {
                    for (b_in, b_out) in t.audio_through.iter() {
                        if !self.buffer_instances.insert(b_in.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b_in.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                        if !self.buffer_instances.insert(b_out.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b_out.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }
                    if let Some((b_in, b_out)) = &t.note_through {
                        if !self.buffer_instances.insert(b_in.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b_in.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                        if !self.buffer_instances.insert(b_out.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b_out.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }

                    for b in t.clear_audio_out.iter() {
                        if !self.buffer_instances.insert(b.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }
                    for b in t.clear_note_out.iter() {
                        if !self.buffer_instances.insert(b.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }
                    if let Some(b) = &t.clear_automation_out {
                        if !self.buffer_instances.insert(b.id()) {
                            return Err(VerifyScheduleError::BufferAppearsTwiceInSameTask {
                                buffer_id: b.id(),
                                task_info: format!("{:?}", &task),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
