use std::fmt::Debug;

use super::schedule::ProcInfo;
use super::ProcBuffers;
use crate::backend::timeline::TimelineTransport;

pub trait AudioGraphNode: Send + Sync {
    /// The number of available mono audio input ports in this node.
    ///
    /// This must remain constant for the lifetime of this node.
    ///
    /// By default, this returns 0 (no ports)
    fn mono_audio_in_ports(&self) -> u32 {
        0
    }

    /// The number of available mono audio output ports in this node.
    ///
    /// This must remain constant for the lifetime of this node.
    ///
    /// By default, this returns 0 (no ports)
    fn mono_audio_out_ports(&self) -> u32 {
        0
    }

    /// The number of available stereo audio input ports in this node.
    ///
    /// This must remain constant for the lifetime of this node.
    ///
    /// By default, this returns 0 (no ports)
    fn stereo_audio_in_ports(&self) -> u32 {
        0
    }

    /// The number of available stereo audio output ports in this node.
    ///
    /// This must remain constant for the lifetime of this node.
    ///
    /// By default, this returns 0 (no ports)
    fn stereo_audio_out_ports(&self) -> u32 {
        0
    }

    /// The delay in audio frames that this node produces.
    ///
    /// This must remain constant for the lifetime of this node.
    ///
    /// By default, this returns 0 (no delay)
    fn delay(&self) -> u32 {
        0
    }

    /// Whether or not this node supports processing with `f64` audio buffers.
    ///
    /// This must remain constant for the lifetime of this node.
    ///
    /// By default, this returns `false`.
    fn supports_f64(&self) -> bool {
        false
    }

    /// Process the given buffers.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that case it
    /// just means some ports are disconnected. However, it is up to the node to communicate with the
    /// program on which specific ports are connected/disconnected.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As such, please do
    /// **not** read from the audio output buffers, and make sure that all unused audio output buffers
    /// are manually cleared in this method.
    ///
    /// In addition, the `sample_rate` and `sample_rate_recip` (1.0 / sample_rate) of the stream
    /// is given. These will remain constant for the lifetime of this node, so these are just provided
    /// for convinience.
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f32>,
    );

    /// Process the given buffers.
    ///
    /// The host will only send this if this node has returned `true` for its `supports_f64` method.
    ///
    /// Please note that even if this node has specified that it supports `f64`, it may still decide
    /// to call the regular `f32` version when the host is set to `f32` mode. This is so nodes are
    /// able to implement both `f32` and `f64` DSP if desired. If your node only uses `f64` dsp, then
    /// you must convert the `f32` buffers manually.
    ///
    /// The number of buffers may be less than the number of ports on this node. In that case it
    /// just means some ports are disconnected. However, it is up to the node to communicate with the
    /// program on which specific ports are connected/disconnected.
    ///
    /// Also please note that the audio output buffers may not be cleared to 0.0. As such, please do
    /// **not** read from the audio output buffers, and make sure that all unused audio output buffers
    /// are manually cleared in this method.
    ///
    /// In addition, the `sample_rate` and `sample_rate_recip` (1.0 / sample_rate) of the stream
    /// is given. These will remain constant for the lifetime of this node, so these are just provided
    /// for convinience.
    #[allow(unused_variables)]
    fn process_f64(
        &mut self,
        proc_info: &ProcInfo,
        transport: &TimelineTransport,
        buffers: &mut ProcBuffers<f64>,
    ) {
    }

    // TODO: Process replacing?
}

// Lets us use unwrap.
impl Debug for Box<dyn AudioGraphNode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Audio Graph Node")
    }
}
