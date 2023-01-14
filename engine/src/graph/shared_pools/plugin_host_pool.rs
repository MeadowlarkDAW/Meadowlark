use audio_graph::NodeID;
use fnv::FnvHashMap;
use meadowlark_plugin_api::PluginInstanceID;

use crate::plugin_host::PluginHostMainThread;

pub(crate) struct PluginHostPool {
    pool: FnvHashMap<u64, PluginHostMainThread>,
    node_id_to_plugin_id: FnvHashMap<NodeID, PluginInstanceID>,
}

impl PluginHostPool {
    pub fn new() -> Self {
        Self { pool: FnvHashMap::default(), node_id_to_plugin_id: FnvHashMap::default() }
    }

    pub fn insert(
        &mut self,
        id: PluginInstanceID,
        host: PluginHostMainThread,
    ) -> Option<PluginHostMainThread> {
        let old_host = self.pool.insert(id.unique_id(), host);
        self.node_id_to_plugin_id.insert(id._node_id().into(), id);
        old_host
    }

    pub fn remove(&mut self, id: &PluginInstanceID) -> Option<PluginHostMainThread> {
        self.node_id_to_plugin_id.remove(&id._node_id().into());
        self.pool.remove(&id.unique_id())
    }

    pub fn get(&self, id: &PluginInstanceID) -> Option<&PluginHostMainThread> {
        self.pool.get(&id.unique_id())
    }

    pub fn get_mut(&mut self, id: &PluginInstanceID) -> Option<&mut PluginHostMainThread> {
        self.pool.get_mut(&id.unique_id())
    }

    pub fn get_by_node_id(&self, id: &NodeID) -> Option<&PluginHostMainThread> {
        self.node_id_to_plugin_id.get(id).map(|id| self.pool.get(&id.unique_id()).unwrap())
    }

    pub fn get_by_unique_id_mut(&mut self, id: u64) -> Option<&mut PluginHostMainThread> {
        self.pool.get_mut(&id)
    }

    pub fn num_plugins(&self) -> usize {
        self.pool.len()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut PluginHostMainThread> {
        self.pool.values_mut()
    }

    pub fn clear(&mut self) {
        self.pool.clear();
        self.node_id_to_plugin_id.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}
