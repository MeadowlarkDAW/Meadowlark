use log::info;
use ringbuf::RingBuffer;
use rusty_daw_io::{ConfigStatus, FatalStreamError, SystemOptions};

mod rt_thread;
mod system_io;
mod ui;

use rt_thread::{MainFatalErrorHandler, MainRtHandler, RtState};

fn main() {
    // Initiate a simple logger that logs all events to the console
    //
    // TODO: Use something more sophisticated
    simple_logger::SimpleLogger::new().init().unwrap();

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
    let (error_tx, mut error_rx) = RingBuffer::<FatalStreamError>::new(1).split(); // Will only ever send one message

    let rt_handler = MainRtHandler::new(rt_state);
    let error_handler = MainFatalErrorHandler::new(error_tx);

    // TODO: Don't panic if starting the stream fails
    let _stream_handle = rusty_daw_io::spawn_rt_thread(
        &config,
        Some(String::from("Meadowlark")),
        rt_handler,
        error_handler,
    )
    .unwrap();

    rt_state_handle.set_msg_channel_active(true);

    // TODO: Use GUI library instead of command line loop
    loop {
        // Check if the stream has crashed
        if let Some(e) = error_rx.pop() {
            // TODO: Don't panic if the audio stream crashes
            panic!("Audio stream crashed: {}", e);
        }

        // TODO process messages from realtime thread

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // Stream automatically closes when `_stream_handle` is dropped, which in turn deallocates
    // everything in `rt_state`
}
