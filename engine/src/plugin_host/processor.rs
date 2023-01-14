use clack_host::events::Event;
use meadowlark_plugin_api::buffer::EventBuffer;
use meadowlark_plugin_api::{PluginProcessor, ProcBuffers, ProcInfo, ProcessStatus};

use crate::utils::thread_id::SharedThreadIDs;

use super::channel::{PlugHostChannelProcThread, PluginActiveState};
use super::event_io_buffers::{PluginEventIoBuffers, PluginEventOutputSanitizer};

// The amount of time to smooth/declick the audio outputs when
// bypassing/unbypassing the plugin.
pub(super) static BYPASS_DECLICK_SECS: f64 = 3.0 / 1000.0;

#[derive(Copy, Clone, Debug, PartialEq)]
enum ProcessingState {
    WaitingForStart,
    Started(ProcessStatus),
    Stopped,
    Errored,
}

pub(crate) struct PluginHostProcessor {
    plugin_processor: Box<dyn PluginProcessor>,
    plugin_instance_id: u64,

    channel: PlugHostChannelProcThread,

    in_events: EventBuffer,
    out_events: EventBuffer,

    event_output_sanitizer: PluginEventOutputSanitizer,

    processing_state: ProcessingState,

    thread_ids: SharedThreadIDs,

    schedule_version: u64,

    bypassed: bool,
    bypass_declick: f32,
    bypass_declick_inc: f32,
    bypass_declick_frames: usize,
    bypass_declick_frames_left: usize,
}

impl PluginHostProcessor {
    pub(crate) fn new(
        plugin_processor: Box<dyn PluginProcessor>,
        plugin_instance_id: u64,
        channel: PlugHostChannelProcThread,
        num_params: usize,
        thread_ids: SharedThreadIDs,
        schedule_version: u64,
        bypass_declick_frames: usize,
    ) -> Self {
        debug_assert_ne!(bypass_declick_frames, 0);

        let bypassed = channel.shared_state.bypassed();
        let bypass_declick = if bypassed { 1.0 } else { 0.0 };
        let bypass_declick_inc = 1.0 / bypass_declick_frames as f32;

        Self {
            plugin_processor,
            plugin_instance_id,
            channel,
            in_events: EventBuffer::with_capacity(num_params * 3),
            out_events: EventBuffer::with_capacity(num_params * 3),
            event_output_sanitizer: PluginEventOutputSanitizer::new(num_params),
            processing_state: ProcessingState::WaitingForStart,
            thread_ids,
            schedule_version,
            bypassed,
            bypass_declick,
            bypass_declick_inc,
            bypass_declick_frames,
            bypass_declick_frames_left: 0,
        }
    }

    /// Returns `true` if gotten a request to drop the processor.
    pub fn process(
        &mut self,
        proc_info: &ProcInfo,
        buffers: &mut ProcBuffers,
        event_buffers: &mut PluginEventIoBuffers,
    ) -> bool {
        let mut do_process = true;

        // Always clear event and note output buffers.
        event_buffers.clear_before_process();

        // --- Read input events from all sources ------------------------------------------------
        // Read input events from all sources and store them in the `in_events`
        // buffer (which is later sent to the plugin's process() or flush() method).

        self.in_events.clear();

        // Read parameter updates from the main thread.
        let mut has_param_in_event = self
            .channel
            .param_queues
            .as_mut()
            .map(|q| q.consume_into_event_buffer(&mut self.in_events))
            .unwrap_or(false);

        // Read parameter automation events from the automation in port.
        let (has_note_in_event, wrote_param_in_event) =
            event_buffers.write_input_events(&mut self.in_events, self.plugin_instance_id);
        has_param_in_event |= wrote_param_in_event;

        // Read transport events from the schedule's transport.
        if let Some(transport_in_event) = proc_info.transport.event() {
            self.in_events.push(transport_in_event.as_unknown());
        }

        // --- Check for requests to drop or start processing ------------------------------------

        // Get the latest activation state of the plugin.
        let state = self.channel.shared_state.get_active_state();

        let mut do_drop = false;
        let mut current_schedule_valid = true;
        if state == PluginActiveState::WaitingToDrop {
            // We got a request to deactivate this plugin, so stop processing
            // and mark this processor to be dropped.

            if let ProcessingState::Started(_) = self.processing_state {
                self.plugin_processor.stop_processing();
            }

            do_process = false;
            do_drop = true;
        } else if self.schedule_version > proc_info.schedule_version {
            // Don't process until the expected schedule arrives. This can happen
            // when a plugin restarting causes the graph to recompile, and that
            // new schedule has not yet arrived.
            do_process = false;
            current_schedule_valid = false;
        } else if self.channel.shared_state.process_requested() {
            // The plugin has requested that we (re)start processing.

            if let ProcessingState::Started(_) = self.processing_state {
            } else {
                self.processing_state = ProcessingState::WaitingForStart;
            }
        }

        let param_flush_requested = self.channel.shared_state.param_flush_requested();

        if self.processing_state == ProcessingState::Errored {
            // We can't process a plugin which failed to start processing.
            do_process = false;
        }

        // --- Check if the plugin should be woken up --------------------------------------------

        // Don't wake up the plugin until the new schedule arrives.
        if current_schedule_valid {
            if let ProcessingState::Stopped | ProcessingState::WaitingForStart =
                self.processing_state
            {
                if self.processing_state == ProcessingState::Stopped && !has_note_in_event {
                    // Check if all audio inputs are silent.

                    // First do a quick check using the constant flags.
                    if buffers.audio_inputs_have_silent_hint() {
                        do_process = false;
                    }

                    if do_process {
                        // If quick check didn't tell us that the buffer was silent,
                        // then do a slow thorough check.
                        if buffers.audio_inputs_silent(proc_info.frames) {
                            do_process = false;
                        }
                    }
                } else if let Err(e) = self.plugin_processor.start_processing() {
                    log::error!("Plugin has failed to start processing: {}", e);

                    // The plugin failed to start processing.
                    self.processing_state = ProcessingState::Errored;

                    do_process = false;
                } else {
                    self.channel.shared_state.set_active_state(PluginActiveState::Active);
                }
            }
        }

        // --- Actual processing -----------------------------------------------------------------

        self.out_events.clear();

        if do_process {
            // Constant flags are opt-in for plugins.
            buffers.set_constant_hint_on_all_outputs(false);

            let new_status =
                if let Some(automation_out_buffer) = &mut event_buffers.automation_out_buffer {
                    let automation_out_buffer = &mut automation_out_buffer.borrow_mut().data;

                    self.plugin_processor.process_with_automation_out(
                        proc_info,
                        buffers,
                        &self.in_events,
                        &mut self.out_events,
                        automation_out_buffer,
                    )
                } else {
                    self.plugin_processor.process(
                        proc_info,
                        buffers,
                        &self.in_events,
                        &mut self.out_events,
                    )
                };

            // --- Update the processing state -------------------------------------------------------

            self.processing_state = match new_status {
                ProcessStatus::Continue => ProcessingState::Started(ProcessStatus::Continue),
                ProcessStatus::ContinueIfNotQuiet => {
                    // Check if the plugin should be put to sleep.
                    if buffers.audio_outputs_have_silent_hint() {
                        self.plugin_processor.stop_processing();
                        ProcessingState::Stopped
                    } else {
                        ProcessingState::Started(ProcessStatus::ContinueIfNotQuiet)
                    }
                }
                ProcessStatus::Tail => {
                    // TODO: handle tail by reading from the tail extension
                    ProcessingState::Started(ProcessStatus::Tail)
                }
                ProcessStatus::Sleep => {
                    self.plugin_processor.stop_processing();

                    ProcessingState::Stopped
                }
                ProcessStatus::Error => {
                    // Discard all output buffers.
                    buffers.clear_all_outputs_and_set_constant_hint(proc_info);
                    ProcessingState::Errored
                }
            };
        } else {
            buffers.clear_all_outputs_and_set_constant_hint(proc_info);

            if state.is_active() && (has_param_in_event || param_flush_requested) {
                self.plugin_processor.param_flush(&self.in_events, &mut self.out_events);
            }
        }

        // --- Read output events from plugin ----------------------------------------------------
        // Read output events from the plugin and store them in the automation out port
        // buffer (if this plugin has one).

        if let Some(params_queue) = &mut self.channel.param_queues {
            // If this plugin has parameters, send parameter updates to the main thread.
            params_queue.to_main_param_value_tx.produce(|mut producer| {
                event_buffers.read_output_events(
                    &self.out_events,
                    Some(&mut producer),
                    &mut self.event_output_sanitizer,
                    proc_info.frames as u32,
                )
            });
        } else {
            event_buffers.read_output_events(
                &self.out_events,
                None,
                &mut self.event_output_sanitizer,
                proc_info.frames as u32,
            );
        }

        // --- Process bypassing/unbypassing the plugin ------------------------------------------

        let bypassed = self.channel.shared_state.bypassed();
        if self.bypassed != bypassed {
            self.bypassed = bypassed;

            if self.bypass_declick_frames_left == 0 {
                self.bypass_declick_frames_left = self.bypass_declick_frames;
                if self.bypassed {
                    self.bypass_declick = 1.0;
                } else {
                    self.bypass_declick = 0.0;
                }
            } else {
                self.bypass_declick_frames_left =
                    self.bypass_declick_frames - self.bypass_declick_frames_left;
            }
        }

        if self.bypass_declick_frames_left != 0 {
            // The plugin is currently in the process of smoothing/declicking the audio
            // output buffers as a result of bypassing/unbypassing the plugin.
            self.bypass_declick(proc_info, buffers);
        } else if self.bypassed {
            // If we didn't process, then the output buffers are already cleared.
            if do_process {
                buffers.clear_all_outputs_and_set_constant_hint(proc_info);
            }

            // The plugin is currently bypassed and has finished smoothing/declicking.
            self.bypass(proc_info, buffers);
        }

        do_drop
    }

    /// The plugin is currently in the process of smoothing/declicking the audio
    /// output buffers as a result of bypassing/unbypassing the plugin.
    fn bypass_declick(&mut self, proc_info: &ProcInfo, buffers: &mut ProcBuffers) {
        let declick_frames = self.bypass_declick_frames_left.min(proc_info.frames);

        let skip_ports = if buffers._main_audio_through_when_bypassed() {
            let main_in_port = &buffers.audio_in[0];
            let main_out_port = &mut buffers.audio_out[0];

            let in_port_iter = main_in_port.iter_f32().unwrap();
            let out_port_iter = main_out_port.iter_f32_mut().unwrap();

            for (in_channel, mut out_channel) in in_port_iter.zip(out_port_iter) {
                let mut declick = self.bypass_declick;

                if self.bypassed {
                    for i in 0..declick_frames {
                        declick -= self.bypass_declick_inc;

                        out_channel.data[i] = (out_channel.data[i] * declick)
                            + (in_channel.data[i] * (1.0 - declick));
                    }
                    if declick_frames < proc_info.frames {
                        out_channel.data[declick_frames..proc_info.frames]
                            .copy_from_slice(&in_channel.data[declick_frames..proc_info.frames]);
                    }
                } else {
                    for i in 0..declick_frames {
                        declick += self.bypass_declick_inc;

                        out_channel.data[i] = (out_channel.data[i] * declick)
                            + (in_channel.data[i] * (1.0 - declick));
                    }
                }

                out_channel.is_constant = false;
            }

            for mut out_channel in
                main_out_port.iter_f32_mut().unwrap().skip(main_in_port.channels())
            {
                let mut declick = self.bypass_declick;

                if self.bypassed {
                    for i in 0..declick_frames {
                        declick -= self.bypass_declick_inc;

                        out_channel.data[i] *= declick;
                    }
                    if declick_frames < proc_info.frames {
                        out_channel.data[declick_frames..proc_info.frames].fill(0.0);
                    }
                } else {
                    for i in 0..declick_frames {
                        declick += self.bypass_declick_inc;

                        out_channel.data[i] *= declick;
                    }
                }

                out_channel.is_constant = false;
            }

            1
        } else {
            0
        };

        for out_port in buffers.audio_out.iter_mut().skip(skip_ports) {
            for mut out_channel in out_port.iter_f32_mut().unwrap() {
                let mut declick = self.bypass_declick;

                if self.bypassed {
                    for i in 0..declick_frames {
                        declick -= self.bypass_declick_inc;

                        out_channel.data[i] *= declick;
                    }
                    if declick_frames < proc_info.frames {
                        out_channel.data[declick_frames..proc_info.frames].fill(0.0);
                    }
                } else {
                    for i in 0..declick_frames {
                        declick += self.bypass_declick_inc;

                        out_channel.data[i] *= declick;
                    }
                }

                out_channel.is_constant = false;
            }
        }

        self.bypass_declick_frames_left -= declick_frames;
        if self.bypassed {
            self.bypass_declick -= self.bypass_declick_inc * declick_frames as f32;
        } else {
            self.bypass_declick += self.bypass_declick_inc * declick_frames as f32;
        }
    }

    /// The plugin is currently bypassed and has finished smoothing/declicking.
    fn bypass(&mut self, proc_info: &ProcInfo, buffers: &mut ProcBuffers) {
        if buffers._main_audio_through_when_bypassed() {
            let main_in_port = &buffers.audio_in[0];
            let main_out_port = &mut buffers.audio_out[0];

            if !main_in_port.has_silent_hint() {
                let in_port_iter = main_in_port.iter_f32().unwrap();
                let out_port_iter = main_out_port.iter_f32_mut().unwrap();

                for (in_channel, mut out_channel) in in_port_iter.zip(out_port_iter) {
                    out_channel.data[0..proc_info.frames]
                        .copy_from_slice(&in_channel.data[0..proc_info.frames]);

                    out_channel.is_constant = in_channel.is_constant;
                }
            }
        }
    }
}

impl Drop for PluginHostProcessor {
    fn drop(&mut self) {
        if self.thread_ids.is_process_thread() {
            if let ProcessingState::Started(_) = self.processing_state {
                self.plugin_processor.stop_processing();
            }
        } else {
            log::error!("Plugin processor was not dropped in the process thread");
        }

        self.channel.shared_state.set_active_state(PluginActiveState::DroppedAndReadyToDeactivate);
    }
}
