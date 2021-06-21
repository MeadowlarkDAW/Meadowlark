use log::info;
use ringbuf::{Consumer, RingBuffer};
// use rusty_daw_io::{
//     ConfigStatus, FatalStreamError, SpawnRtThreadError, StreamHandle, SystemOptions,
// };

pub mod components;

use tuix::*;
use tuix::style::themes::DEFAULT_THEME;

use self::components::LevelsMeter;

const THEME: &str = include_str!("theme.css");

// use crate::rt_thread::{MainFatalErrorHandler, MainRtHandler, RtState};

pub struct App {
    // State could go here
}

impl App {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl Widget for App {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        
        let row = Row::new().build(state, entity, |builder| builder.set_width(Stretch(1.0)).set_height(Stretch(1.0)));
        
        ValueKnob::new("Amplitude", 0.0,0.0,1.0).build(state, row, |builder| 
            builder
                .set_width(Pixels(50.0))
                .set_height(Pixels(50.0))
                .set_space(Stretch(1.0))
        );

        LevelsMeter::new().build(state, row, |builder|
            builder
                .set_height(Pixels(200.0))
                .set_width(Pixels(50.0))
                .set_space(Stretch(1.0))
                .set_background_color(Color::rgb(50, 50, 50))
        );


        entity
    }
}

pub fn run() {
    let window_description = WindowDescription::new().with_title("Meadowlark");
    let app = Application::new(window_description, |state, window| {
        state.add_theme(DEFAULT_THEME);
        state.add_theme(THEME);
        //StreamHandleState::new().build(state, window, |builder| builder);


        App::new().build(state, window, |builder| builder);
    });

    app.run();

    // Stream automatically closes when `stream_handle` is dropped, which in turn deallocates
    // everything in `rt_state`
}


/*
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
*/