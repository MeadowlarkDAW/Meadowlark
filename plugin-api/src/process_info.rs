use smallvec::SmallVec;

use crate::transport::TransportInfo;

use crate::buffer::{AudioPortBuffer, AudioPortBufferMut};

/// The status of a call to a plugin's `process()` method.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Processing failed. The output buffer must be discarded.
    Error = 0,

    /// Processing succeeded, keep processing.
    Continue = 1,

    /// Processing succeeded, keep processing if the output is not quiet.
    ContinueIfNotQuiet = 2,

    /// Rely upon the plugin's tail to determine if the plugin should continue to process.
    /// see clap_plugin_tail
    Tail = 3,

    /// Processing succeeded, but no more processing is required until
    /// the next event or variation in audio input.
    Sleep = 4,
}

pub struct ProcInfo {
    /// A steady sample time counter.
    ///
    /// This field can be used to calculate the sleep duration between two process calls.
    /// This value may be specific to this plugin instance and have no relation to what
    /// other plugin instances may receive.
    ///
    /// This will be `-1` if not available, otherwise the value will be increased by
    /// at least `frames_count` for the next call to process.
    pub steady_time: i64,

    /// The number of frames to process. All buffers in this struct are gauranteed to be
    /// at-least this length.
    pub frames: usize,

    pub transport: TransportInfo,

    /// The version of the compiled schedule. Can be used for debugging purposes.
    pub schedule_version: u64,
}

pub struct ProcBuffers {
    pub audio_in: SmallVec<[AudioPortBuffer; 2]>,
    pub audio_out: SmallVec<[AudioPortBufferMut; 2]>,

    main_audio_through_when_bypassed: bool,
}

impl ProcBuffers {
    pub fn _new(
        audio_in: SmallVec<[AudioPortBuffer; 2]>,
        audio_out: SmallVec<[AudioPortBufferMut; 2]>,
        main_audio_through_when_bypassed: bool,
    ) -> Self {
        Self { audio_in, audio_out, main_audio_through_when_bypassed }
    }

    /// Checks if all audio input buffers are silent for a given number of frames, i.e. if all
    /// sample values are equal to `0`.
    pub fn audio_inputs_silent(&self, frames: usize) -> bool {
        for buf in self.audio_in.iter() {
            if !buf.is_silent(frames) {
                return false;
            }
        }
        true
    }

    /// Checks if all audio output buffers are silent for a given number of frames, i.e. if all
    /// sample values are equal to `0`.
    pub fn audio_outputs_silent(&self, frames: usize) -> bool {
        for buf in self.audio_out.iter() {
            if !buf.is_silent(frames) {
                return false;
            }
        }
        true
    }

    /// Checks if all audio input buffers could be possibly silent, without reading the whole buffer.
    ///
    /// This only relies on the `is_constant` flag and the first sample of each buffer, and thus
    /// may not be accurate.
    pub fn audio_inputs_have_silent_hint(&self) -> bool {
        self.audio_in.iter().all(|b| b.has_silent_hint())
    }

    /// Checks if all audio output buffers could be possibly silent, without reading the whole buffer.
    ///
    /// This only relies on the `is_constant` flag and the first sample of each buffer, and thus
    /// may not be accurate.
    pub fn audio_outputs_have_silent_hint(&self) -> bool {
        self.audio_out.iter().all(|b| b.has_silent_hint())
    }

    /// Clear all output buffers.
    ///
    /// All output buffers which have not already been filled manually must be cleared.
    ///
    /// Note this does not set the constant hint.
    pub fn clear_all_outputs(&mut self, proc_info: &ProcInfo) {
        for buf in self.audio_out.iter_mut() {
            buf.clear_all(proc_info.frames);
        }
    }

    /// Clear all output buffers while also setting the constant hint to `true` on all
    /// output buffers.
    ///
    /// All output buffers which have not already been filled manually must be cleared.
    pub fn clear_all_outputs_and_set_constant_hint(&mut self, proc_info: &ProcInfo) {
        for buf in self.audio_out.iter_mut() {
            buf.clear_all_and_set_constant_hint(proc_info.frames);
        }
    }

    pub fn set_constant_hint_on_all_outputs(&mut self, is_constant: bool) {
        for buf in self.audio_out.iter_mut() {
            buf.set_constant_hint(is_constant);
        }
    }

    pub fn _main_audio_through_when_bypassed(&self) -> bool {
        self.main_audio_through_when_bypassed
    }

    /// Copy all inputs to their corresponding outputs, and clear all other outputs.
    pub fn bypassed(&mut self, proc_info: &ProcInfo) {
        self.clear_all_outputs(proc_info);

        // TODO: More audio through ports when bypassed?
        if self.main_audio_through_when_bypassed {
            let main_in_port = &self.audio_in[0];
            let main_out_port = &mut self.audio_out[0];

            if !main_in_port.has_silent_hint() {
                let in_port_iter = if let Some(i) = main_in_port.iter_f32() {
                    i
                } else {
                    return;
                };
                let out_port_iter = if let Some(i) = main_out_port.iter_f32_mut() {
                    i
                } else {
                    return;
                };

                for (in_channel, mut out_channel) in in_port_iter.zip(out_port_iter) {
                    out_channel.data[0..proc_info.frames]
                        .copy_from_slice(&in_channel.data[0..proc_info.frames]);

                    out_channel.is_constant = in_channel.is_constant;
                }
            }
        }
    }
}
