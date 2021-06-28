use atomic_refcell::{AtomicRef, AtomicRefMut};
use std::fmt::Debug;

use super::resource_pool::{MonoAudioPortBuffer, StereoAudioPortBuffer};
use super::schedule::ProcInfo;

pub const MAX_AUDIO_IN_PORTS: usize = 64;
pub const MAX_AUDIO_OUT_PORTS: usize = 64;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioPortType {
    Mono,
    Stereo,
}

pub trait AudioGraphNode: Send + Sync {
    /// The number of available mono audio input ports in this node.
    ///
    /// This must always remain constant for every Node of this type. We can't just
    /// make this a constant in the trait because we need to bind it to a vtable.
    ///
    /// The number of ports (not channels) cannot exceed `MAX_AUDIO_IN_PORTS` (64)
    ///
    /// By default, this returns 0 (no ports)
    fn mono_audio_in_ports(&self) -> usize {
        0
    }

    /// The number of available mono audio output ports in this node.
    ///
    /// This must always remain constant for every Node of this type. We can't just
    /// make this a constant in the trait because we need to bind it to a vtable.
    ///
    /// The number of ports (not channels) cannot exceed `MAX_AUDIO_OUT_PORTS` (64)
    ///
    /// By default, this returns 0 (no ports)
    fn mono_audio_out_ports(&self) -> usize {
        0
    }

    /// The number of available stereo audio input ports in this node.
    ///
    /// This must always remain constant for every Node of this type. We can't just
    /// make this a constant in the trait because we need to bind it to a vtable.
    ///
    /// The number of ports (not channels) cannot exceed `MAX_AUDIO_IN_PORTS` (64)
    ///
    /// By default, this returns 0 (no ports)
    fn stereo_audio_in_ports(&self) -> usize {
        0
    }

    /// The number of available stereo audio output ports in this node.
    ///
    /// This must always remain constant for every Node of this type. We can't just
    /// make this a constant in the trait because we need to bind it to a vtable.
    ///
    /// The number of ports (not channels) cannot exceed `MAX_AUDIO_OUT_PORTS` (64)
    ///
    /// By default, this returns 0 (no ports)
    fn stereo_audio_out_ports(&self) -> usize {
        0
    }

    /// Process the given buffers.
    ///
    /// The scheduler will uphold several gaurantees with these buffers:
    ///
    /// * `mono_audio_in` will always contain `Self::mono_audio_in_ports()` ports.
    /// * `mono_audio_out` will always contain `Self::mono_audio_out_ports()` ports.
    /// * `stereo_audio_in` will always contain `Self::stereo_audio_in_ports()` ports.
    /// * `stereo_audio_out` will always contain `Self::stereo_audio_out_ports()` ports.
    ///
    /// In addition, the `sample_rate` and `sample_rate_recip` (1.0 / sample_rate) of the stream
    /// is given. These will remain constant for the lifetime of this node, so these are just provided
    /// for convinience.
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        mono_audio_in: &[AtomicRef<MonoAudioPortBuffer>],
        mono_audio_out: &mut [AtomicRefMut<MonoAudioPortBuffer>],
        stereo_audio_in: &[AtomicRef<StereoAudioPortBuffer>],
        stereo_audio_out: &mut [AtomicRefMut<StereoAudioPortBuffer>],
    );
}

// Lets us use unwrap.
impl Debug for Box<dyn AudioGraphNode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Audio Graph Node")
    }
}
