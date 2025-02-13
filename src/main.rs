pub static MEADOWLARK_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const IS_NIGHTLY: bool = false;

mod state;
mod ui;

fn main() -> anyhow::Result<()> {
    setup_logging();

    ui::run()
}

fn setup_logging() {
    use log::LevelFilter;

    let mut env_builder = env_logger::builder();

    env_builder.filter_level(if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    });

    // Override the filter level from the environment if one is
    // specified.
    env_builder.parse_default_env();

    // Disable logging in crates which produce a lot of noise.
    env_builder.filter_module("selectors::matching", LevelFilter::Info);
    env_builder.filter_module("vizia_core", LevelFilter::Info);
    env_builder.filter_module("sctk", LevelFilter::Info);

    env_builder.init();
}
