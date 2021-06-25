use basedrop::{Collector, Handle, Shared, SharedCell};

pub mod buffer_pool;
pub mod node;
pub mod node_pool;
pub mod schedule;

use buffer_pool::BufferPool;
use node_pool::NodePool;
use schedule::Schedule;

pub struct FrontendState {
    shared_state: Shared<SharedCell<SharedState>>,

    collector: Collector,
}

impl FrontendState {
    pub fn new() -> (Self, Shared<SharedCell<SharedState>>) {
        let collector = Collector::new();

        let shared_state = SharedState::new(&collector.handle());
        let rt_shared_state = Shared::clone(&shared_state);

        (
            Self {
                shared_state,
                collector,
            },
            rt_shared_state,
        )
    }

    /// A temporary test setup: "sine wave generator" -> "gain knob" -> "db meter".
    pub fn test_setup(&mut self) {}

    /// Call periodically to collect garbage in the rt thread.
    pub fn collect(&mut self) {
        self.collector.collect();
    }
}

pub struct SharedState {
    pub buffer_pool: Shared<BufferPool>,
    pub node_pool: Shared<NodePool>,
}

impl SharedState {
    fn new(coll_handle: &Handle) -> Shared<SharedCell<SharedState>> {
        Shared::new(
            coll_handle,
            SharedCell::new(Shared::new(
                coll_handle,
                SharedState {
                    buffer_pool: Shared::new(coll_handle, BufferPool::new()),
                    node_pool: Shared::new(coll_handle, NodePool::new()),
                },
            )),
        )
    }
}
