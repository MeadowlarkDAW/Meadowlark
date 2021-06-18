use log::info;
use ringbuf::{Consumer, RingBuffer};
use rusty_daw_io::{
    ConfigStatus, FatalStreamError, SpawnRtThreadError, StreamHandle, SystemOptions,
};

use tuix::events::{BuildHandler, EventHandler};
use tuix::style::themes::DEFAULT_THEME;
use tuix::widgets::Button;
use tuix::{Application, Entity, Event, PropSet, State};

use crate::rt_thread::{MainFatalErrorHandler, MainRtHandler, RtState};

pub fn run() {
    let app = Application::new(|win_desc, state, window| {
        state.add_theme(DEFAULT_THEME);

        StreamHandleState::new().build(state, window, |builder| builder);

        Button::new().build(state, window, |builder| builder.set_text("Button"));

        win_desc.with_title("Meadowlark")
    });

    app.run();

    // Stream automatically closes when `stream_handle` is dropped, which in turn deallocates
    // everything in `rt_state`
}

struct StreamHandleState {
    stream_handle: Option<StreamHandle<MainRtHandler, MainFatalErrorHandler>>,
    error_rx: Option<Consumer<FatalStreamError>>,
}

impl StreamHandleState {
    pub fn new() -> Self {
        // TODO: Don't panic if spawning stream fails.
        let (stream_handle, error_rx) = load_default_stream().unwrap();

        Self {
            stream_handle: Some(stream_handle),
            error_rx: Some(error_rx),
        }
    }
}

impl BuildHandler for StreamHandleState {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        entity.set_element(state, "stream_handle_state")
    }
}

impl EventHandler for StreamHandleState {
    fn on_event(&mut self, state: &mut State, entity: Entity, event: &mut Event) {}
}

fn load_default_stream() -> Result<
    (
        StreamHandle<MainRtHandler, MainFatalErrorHandler>,
        Consumer<FatalStreamError>,
    ),
    SpawnRtThreadError,
> {
    let system_opts = SystemOptions::new();

    // The config should have the default settings on startup. Use that for now
    // TODO: Audio/Midi settings UI
    let config = match system_opts.config_status() {
        ConfigStatus::Ok {
            config,
            sample_rate,
            latency_frames,
            latency_ms,
        } => {
            info!("Loaded default io config: {:?}", config);
            info!(
                "sample rate: {} | latency: {} frames ({:.1} ms)",
                sample_rate, latency_frames, latency_ms
            );
            config
        }
        // TODO: Don't panic if default config fails
        ConfigStatus::AudioServerUnavailable(s) => {
            panic!(
                "Could not load default config. The {} audio server is unavailable",
                s
            );
        }
        ConfigStatus::NoAudioDeviceAvailable => {
            panic!("Could not load default config. No compatible audio device is available");
        }
        ConfigStatus::UnknownError => {
            panic!("Could not load default config. An unkown error occurred");
        }
    };

    // The state of the realtime thread, as well as a handle to that state which syncs state using
    // message channels.
    let (rt_state, mut rt_state_handle) = RtState::new();

    // Create a message channel that listens to fatal stream errors (when the stream crashes)
    let (error_tx, error_rx) = RingBuffer::<FatalStreamError>::new(1).split(); // Will only ever send one message

    let rt_handler = MainRtHandler::new(rt_state);
    let error_handler = MainFatalErrorHandler::new(error_tx);

    let stream_handle = rusty_daw_io::spawn_rt_thread(
        &config,
        Some(String::from("Meadowlark")),
        rt_handler,
        error_handler,
    )?;

    rt_state_handle.set_msg_channel_active(true);

    Ok((stream_handle, error_rx))
}
