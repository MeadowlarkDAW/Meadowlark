pub mod error;

mod channel;
mod main_thread;
mod processor;
mod save_state;

pub(crate) mod event_io_buffers;
pub(crate) mod external;

pub use main_thread::{ParamModifiedInfo, ParamState, PluginHostMainThread};
pub use save_state::PluginHostSaveState;

pub(crate) use channel::{PluginHostProcessorWrapper, SharedPluginHostProcessor};
pub(crate) use main_thread::OnIdleResult;
