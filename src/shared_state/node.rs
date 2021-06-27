use std::fmt::Debug;

use basedrop::Shared;

use super::schedule::ProcInfo;

pub trait AudioGraphNode: Send + Sync {
    /// The number of audio through ports (process_replacing)
    ///
    /// This must always remain constant for every Node of this type. We can't just
    /// make this a constant in the trait because we need to bind it to a vtable.
    fn audio_through_ports(&self) -> usize;

    /// The number of extra audio input ports (not including any "audio through" ports).
    ///
    /// This is `0` by default.
    ///
    /// This must always remain constant for every Node of this type. We can't just
    /// make this a constant in the trait because we need to bind it to a vtable.
    fn extra_audio_in_ports(&self) -> usize {
        0
    }

    /// The number of extra audio output ports (not including any "audio through" ports)
    ///
    /// This is `0` by default.
    ///
    /// This must always remain constant for every Node of this type. We can't just
    /// make this a constant in the trait because we need to bind it to a vtable.
    fn extra_audio_out_ports(&self) -> usize {
        0
    }

    /// Process the given buffers.
    ///
    /// The scheduler will uphold several gaurantees with these buffers:
    ///
    /// * `audio_through` will always contain `Self::audio_through_ports()` buffers.
    /// * `extra_audio_in` will always contain `Self::extra_audio_in_ports()` buffers.
    /// * `extra_audio_out` will always contain `Self::extra_audio_out_ports()` buffers.
    /// * The length of every audio buffer Vec will be the same as `frames`.
    ///
    /// Because of this, unchecked array indexing in these buffers may be used if better
    /// looping performance is needed.
    ///
    /// TODO: Find some way to uphold this unchecked array indexing safety more ergonomically
    /// using a custom type?
    ///
    /// In addition, the `sample_rate` and `sample_rate_recip` (1.0 / sample_rate) of the stream
    /// is given. These will remain constant for the lifetime of this node, so these are just provided
    /// for convinience.
    fn process(
        &mut self,
        proc_info: &ProcInfo,
        audio_through: &mut Vec<Shared<Vec<f32>>>,
        extra_audio_in: &Vec<Shared<Vec<f32>>>,
        extra_audio_out: &mut Vec<Shared<Vec<f32>>>,
    );
}

// Lets us use unwrap.
impl Debug for Box<dyn AudioGraphNode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Audio Graph Node")
    }
}
