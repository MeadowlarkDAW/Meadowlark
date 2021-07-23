pub mod cpu_id;
pub mod generic_nodes;
pub mod graph_interface;
pub mod hardware_io;
pub mod parameter;
pub mod project_interface;
pub mod resource_loader;
pub mod rt_thread;
pub mod timeline;

pub use parameter::{coeff_to_db, db_to_coeff};

pub use project_interface::{ProjectInterface, ProjectSaveState};

pub const MAX_BLOCKSIZE: usize = 128;
