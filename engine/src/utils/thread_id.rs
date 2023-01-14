use basedrop::{Shared, SharedCell};
use std::thread::ThreadId;

pub(crate) struct SharedThreadIDs {
    // TODO: Use AtomicU64 instead once ThreadId::as_u64() becomes stable?
    main_thread_id: Shared<SharedCell<Option<ThreadId>>>,
    process_thread_id: Shared<SharedCell<Option<ThreadId>>>,
}

impl Clone for SharedThreadIDs {
    fn clone(&self) -> Self {
        Self {
            main_thread_id: Shared::clone(&self.main_thread_id),
            process_thread_id: Shared::clone(&self.process_thread_id),
        }
    }
}

impl SharedThreadIDs {
    pub fn new(
        main_thread_id: Option<ThreadId>,
        process_thread_id: Option<ThreadId>,
        coll_handle: &basedrop::Handle,
    ) -> Self {
        Self {
            main_thread_id: Shared::new(
                coll_handle,
                SharedCell::new(Shared::new(coll_handle, main_thread_id)),
            ),
            process_thread_id: Shared::new(
                coll_handle,
                SharedCell::new(Shared::new(coll_handle, process_thread_id)),
            ),
        }
    }

    pub fn main_thread_id(&self) -> Option<ThreadId> {
        *self.main_thread_id.get()
    }

    pub fn process_thread_id(&self) -> Option<ThreadId> {
        *self.process_thread_id.get()
    }

    pub fn is_main_thread(&self) -> bool {
        if let Some(main_thread_id) = *self.main_thread_id.get() {
            std::thread::current().id() == main_thread_id
        } else {
            false
        }
    }

    pub fn is_process_thread(&self) -> bool {
        if let Some(process_thread_id) = *self.process_thread_id.get() {
            std::thread::current().id() == process_thread_id
        } else {
            false
        }
    }

    pub fn set_process_thread_id(&self, id: ThreadId, coll_handle: &basedrop::Handle) {
        self.process_thread_id.set(Shared::new(coll_handle, Some(id)));
    }
}
