pub static DEFAULT_IDLE_INTERVAL_MS: u32 = 16;
pub static DEFAULT_GARBAGE_COLLECT_INTERVAL_MS: u32 = 3_000;
pub static DEFAULT_TRANSPORT_DECLICK_SECONDS: f64 = 3.0 / 1_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineSettings {
    pub main_idle_interval_ms: u32,
    pub garbage_collect_interval_ms: u32,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            main_idle_interval_ms: DEFAULT_IDLE_INTERVAL_MS,
            garbage_collect_interval_ms: DEFAULT_GARBAGE_COLLECT_INTERVAL_MS,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ActivateEngineSettings {
    /// The sample rate of the project.
    pub sample_rate: u32,

    /// The minimum number of frames (samples in a single audio channel)
    /// the can be processed in a single process cycle.
    pub min_frames: u32,

    /// The maximum number of frames (samples in a single audio channel)
    /// the can be processed in a single process cycle.
    pub max_frames: u32,

    /// The total number of input audio channels to the audio graph.
    pub num_audio_in_channels: u16,

    /// The total number of output audio channels from the audio graph.
    pub num_audio_out_channels: u16,

    /// The pre-allocated capacity for note buffers in the audio graph.
    ///
    /// By default this is set to `256`.
    pub note_buffer_size: usize,

    /// The pre-allocated capacity for parameter event buffers in the audio
    /// graph.
    ///
    /// By default this is set to `256`.
    pub event_buffer_size: usize,

    /// The time window for the transport's declick buffers.
    ///
    /// By default this is set to 3ms.
    pub transport_declick_seconds: f64,

    /// If true, all audio output buffers will be hard clipped at 0dB.
    ///
    /// By default this is set to `false`.
    pub hard_clip_outputs: bool,
}

impl Default for ActivateEngineSettings {
    fn default() -> Self {
        Self {
            sample_rate: 44_100,
            min_frames: 1,
            max_frames: 512,
            num_audio_in_channels: 2,
            num_audio_out_channels: 2,
            note_buffer_size: 256,
            event_buffer_size: 256,
            transport_declick_seconds: DEFAULT_TRANSPORT_DECLICK_SECONDS,
            hard_clip_outputs: false,
        }
    }
}
