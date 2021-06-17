use basedrop::Owned;

pub mod resource_pool;
pub use resource_pool::{PcmResource, PcmResourceId};

use resource_pool::{ResourcePool, ResourcePoolMsg, ResourcePoolUiHandle};

pub struct RtState {
    resource_pool: ResourcePool,

    from_handle_rx: llq::Consumer<Option<HandleToRtMsg>>,

    // Channel that sends back message nodes to be re-used by the Ui handle, to avoid
    // the Handle having to constantly allocate and de-allocate new message nodes.
    handle_msg_pool_tx: llq::Producer<Option<HandleToRtMsg>>,

    msg_channel_active: bool,
    // TODO: Rt to Ui handle communication
}

impl RtState {
    pub fn new() -> (Self, RtStateUiHandle) {
        let (resource_pool, resource_pool_handle) = ResourcePool::new();

        let (to_rt_tx, from_handle_rx) = llq::Queue::<Option<HandleToRtMsg>>::new().split();

        // Channel that sends back message nodes to be re-used by the Ui handle, to avoid
        // the Handle having to constantly allocate and de-allocate new message nodes.
        let (handle_msg_pool_tx, handle_msg_pool_rx) =
            llq::Queue::<Option<HandleToRtMsg>>::new().split();

        (
            Self {
                resource_pool,
                from_handle_rx,
                handle_msg_pool_tx,
                msg_channel_active: false,
            },
            RtStateUiHandle::new(to_rt_tx, handle_msg_pool_rx, resource_pool_handle),
        )
    }

    pub fn sync(&mut self) {
        while let Some(mut msg_node) = self.from_handle_rx.pop() {
            if let Some(msg) = msg_node.take() {
                match msg {
                    HandleToRtMsg::ResourcePool(resource_pool_msg) => {
                        match resource_pool_msg {
                            ResourcePoolMsg::AddPcmResource((resource, id)) => {
                                self.resource_pool.add_pcm(resource, id)
                            }
                            ResourcePoolMsg::RemovePcmResource(id) => {
                                self.resource_pool.remove_pcm(id);

                                // IMPORTANT: Make sure all references to this resource are deleted!
                            }
                        }
                    }

                    HandleToRtMsg::MsgChannelActive(active) => self.msg_channel_active = active,
                }
            }

            // Send the message node back to the handle to be reused
            self.handle_msg_pool_tx.push(msg_node);
        }
    }
}

pub struct RtStateUiHandle {
    resource_pool: ResourcePoolUiHandle,

    to_rt_tx: llq::Producer<Option<HandleToRtMsg>>,

    // Buffer that collects used messages so they can be reused. This avoids
    // the Handle having to constantly allocate and de-allocate new messages.
    handle_msg_pool_rx: llq::Consumer<Option<HandleToRtMsg>>,

    // TODO: Rt thread to Handle communication
    collector: basedrop::Collector,

    msg_channel_active: bool,
}

impl RtStateUiHandle {
    fn new(
        to_rt_tx: llq::Producer<Option<HandleToRtMsg>>,
        handle_msg_pool_rx: llq::Consumer<Option<HandleToRtMsg>>,
        resource_pool: ResourcePoolUiHandle,
    ) -> Self {
        Self {
            to_rt_tx,
            resource_pool,
            collector: basedrop::Collector::new(),
            handle_msg_pool_rx,
            msg_channel_active: false,
        }
    }

    fn send(&mut self, msg: HandleToRtMsg) {
        // Try to re-use an old allocated message slot
        let mut msg_node = self
            .handle_msg_pool_rx
            .pop()
            .unwrap_or(llq::Node::new(None));

        let _ = msg_node.replace(msg);

        self.to_rt_tx.push(msg_node);
    }

    pub fn sync_from_rt(&mut self) {
        self.collector.collect();
    }

    pub fn set_msg_channel_active(&mut self, active: bool) {
        if self.msg_channel_active != active {
            self.msg_channel_active = active;
            self.send(HandleToRtMsg::MsgChannelActive(active));
        }
    }

    pub fn msg_channel_active(&self) -> bool {
        self.msg_channel_active
    }

    pub fn add_pcm_resource(
        &mut self,
        resource: Box<dyn PcmResource>,
    ) -> Result<PcmResourceId, ()> {
        if let Ok(id) = self.resource_pool.add_new_pcm() {
            self.send(HandleToRtMsg::ResourcePool(
                ResourcePoolMsg::AddPcmResource((
                    Owned::new(&self.collector.handle(), resource),
                    id,
                )),
            ));
            Ok(id)
        } else {
            Err(())
        }
    }

    pub fn remove_pcm_resource(&mut self, id: PcmResourceId) {
        self.resource_pool.remove_pcm_id(id);
        self.send(HandleToRtMsg::ResourcePool(
            ResourcePoolMsg::RemovePcmResource(id),
        ));
    }
}

#[derive(Debug, Clone)]
enum RtToHandleMsg {}

enum HandleToRtMsg {
    ResourcePool(ResourcePoolMsg),

    /// Tells the rt thread that it should start or stop sending any messages to the UI
    MsgChannelActive(bool),
}
