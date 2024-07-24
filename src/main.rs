mod config;
mod gui;
mod state;

pub use self::state::Action;

pub static MEADOWLARK_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const IS_NIGHTLY: bool = false;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    // Actions are sent via a regular Rust mpsc queue.
    let (action_sender, action_receiver) = yarrow::action_channel();

    yarrow::run_blocking(
        self::state::App::new(action_sender.clone(), action_receiver),
        action_sender,
    )
}

fn setup_logging() {
    use log::LevelFilter;

    #[cfg(debug_assertions)]
    // The default level filter in debug mode
    const DEFAULT_LEVEL: LevelFilter = LevelFilter::Debug;
    #[cfg(not(debug_assertions))]
    // The default level filter in release mode
    const DEFAULT_LEVEL: LevelFilter = LevelFilter::Info;

    let mut env_builder = env_logger::builder();
    env_builder.filter_level(DEFAULT_LEVEL);

    // Override the filter level from the environment if one is
    // specified.
    env_builder.parse_default_env();

    // Silence dependencies that produce a lot of noise.
    env_builder.filter_module("naga", LevelFilter::Info);
    env_builder.filter_module("wgpu_hal", LevelFilter::Warn);
    env_builder.filter_module("wgpu_core::device", LevelFilter::Info);
    env_builder.filter_module("wgpu_core::device::resource", LevelFilter::Warn);
    env_builder.filter_module("wgpu_core::present", LevelFilter::Info);
    env_builder.filter_module("wgpu_core::resource", LevelFilter::Info);
    env_builder.filter_module("sctk", LevelFilter::Info);
    env_builder.filter_module("cosmic_text::buffer", LevelFilter::Info);
    env_builder.filter_module("cosmic_text::font::system", LevelFilter::Info);

    env_builder.init();
}
