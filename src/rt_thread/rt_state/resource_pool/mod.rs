use basedrop::Owned;

pub mod id;
pub mod pcm_resource;

pub use pcm_resource::PcmResource;

pub const PCM_RESOURCE_POOL_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcmResourceId(usize);

pub struct ResourcePool {
    // TODO: I'm not sure how much I like this "using an array index as an ID" system. It limits
    // us to using a fixed maximum number of resources, and it requires the UI to make sure it
    // correctly removes all of its own copied IDs when one is deleted.

    // TODO: Have some way that lets the UI read these PCM resources, so it doesn't need to keep a copy
    // for itself? These PCM resources are immutable by design, so we could probably use basedrop::Shared.
    pcm_resources: [Option<Owned<Box<dyn PcmResource>>>; PCM_RESOURCE_POOL_SIZE],
}

impl ResourcePool {
    pub(super) fn new() -> (Self, ResourcePoolUiHandle) {
        const INIT: Option<Owned<Box<dyn PcmResource>>> = None;
        (
            Self {
                pcm_resources: [INIT; PCM_RESOURCE_POOL_SIZE],
            },
            ResourcePoolUiHandle::new(),
        )
    }

    pub(super) fn add_pcm(&mut self, resource: Owned<Box<dyn PcmResource>>, id: PcmResourceId) {
        self.pcm_resources[id.0] = Some(resource)
    }

    pub(super) fn remove_pcm(&mut self, id: PcmResourceId) {
        self.pcm_resources[id.0] = None;
    }

    pub fn get_pcm(&self, handle: PcmResourceId) -> Result<&Owned<Box<dyn PcmResource>>, ()> {
        if let Some(r) = &self.pcm_resources[handle.0] {
            Ok(r)
        } else {
            Err(())
        }
    }
}

pub struct ResourcePoolUiHandle {
    pcm_resource_state: [bool; PCM_RESOURCE_POOL_SIZE],

    // TODO: Make this a hashmap that maps file names to the resource ID
    pcm_resource_ids: Vec<PcmResourceId>,
}

impl ResourcePoolUiHandle {
    fn new() -> Self {
        Self {
            pcm_resource_state: [false; PCM_RESOURCE_POOL_SIZE],
            pcm_resource_ids: Vec::new(),
        }
    }

    pub(super) fn add_new_pcm(&mut self) -> Result<PcmResourceId, ()> {
        // Find the next available ID available.
        //
        // TODO: Optimize?
        let mut index = 0;
        let mut found = false;
        for r in self.pcm_resource_state.iter() {
            if !*r {
                found = true;
                break;
            }
            index += 1;
        }
        if found {
            let id = PcmResourceId(index);
            self.pcm_resource_state[index] = true;
            self.pcm_resource_ids.push(id);
            Ok(id)
        } else {
            Err(())
        }
    }

    pub(super) fn remove_pcm_id(&mut self, id: PcmResourceId) {
        let mut i = 0;
        let mut found = false;
        for r in self.pcm_resource_ids.iter() {
            if r.0 == id.0 {
                found = true;
                break;
            }
        }
        if found {
            self.pcm_resource_ids.remove(i);
        }
        self.pcm_resource_state[id.0] = false;
    }

    pub fn current_pcm_ids(&self) -> &[PcmResourceId] {
        &self.pcm_resource_ids
    }
}

pub enum ResourcePoolMsg {
    AddPcmResource((Owned<Box<dyn PcmResource>>, PcmResourceId)),
    RemovePcmResource(PcmResourceId),
}
