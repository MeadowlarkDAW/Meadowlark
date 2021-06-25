pub mod gain;
pub mod monitor;
pub mod sine_gen;

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
    fn process(&mut self, proc_info: ProcInfo);
}

pub struct ProcInfo<'a> {
    /// The number of frames in every audio buffer.
    frames: usize,

    /// The audio through (process_replacing) buffers.
    ///
    /// The scheuler will uphold these guarantees:
    ///
    /// * The number of buffers will always equal `Self::audio_through_ports()`.
    /// * Each buffer will always have the length `frames`.
    audio_through: &'a mut [&'a mut Vec<f32>],

    /// The extra audio input buffers (not including any "audio through" buffers).
    ///
    /// The scheuler will uphold these guarantees:
    ///
    /// * The number of buffers will always equal `Self::extra_audio_in_ports()`.
    /// * Each buffer will always have the length `frames`.
    extra_audio_in: &'a [&'a Vec<f32>],

    /// The extra audio output buffers (not including any "audio through" buffers).
    ///
    /// The scheuler will uphold these guarantees:
    ///
    /// * The number of buffers will always equal `Self::extra_audio_out_ports()`.
    /// * Each buffer will always have the length `frames`.
    extra_audio_out: &'a mut [&'a mut Vec<f32>],

    /// The sample rate of the stream. This remains constant for the whole lifetime of this node,
    /// so this is just provided for convenience.
    sample_rate: f32,

    /// The recipricocl of the sample rate (1.0 / sample_rate) of the stream. This remains constant
    /// for the whole lifetime of this node, so this is just provided for convenience.
    sample_rate_recip: f32,
}
