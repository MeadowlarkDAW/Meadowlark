use basedrop::Shared;
use std::ops::Range;

use super::node::AudioGraphNode;

pub struct NodePool {
    pub nodes: Vec<Shared<Box<dyn AudioGraphNode>>>,
}

impl NodePool {
    pub(super) fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(super) fn add_new_nodes(&mut self, mut new_nodes: Vec<Shared<Box<dyn AudioGraphNode>>>) {
        // Create a new Vec of pointers to the nodes.
        let mut new_vec = Vec::<Shared<Box<dyn AudioGraphNode>>>::with_capacity(
            self.nodes.len() + new_nodes.len(),
        );

        // Clone the old Vec.
        for node in self.nodes.iter() {
            new_vec.push(Shared::clone(node));
        }

        // Add the new nodes.
        new_vec.append(&mut new_nodes);

        // Our reference to the old Vec of pointers is dropped here and is replaced with the new one.
        // The old Vec will be fully dropped once the rt thread finishes using it.
        self.nodes = new_vec;
    }

    /// Remove nodes from the pool.
    ///
    /// * `range` - The range of indexes (`start <= x < end`) of the nodes to remove.
    ///
    /// This will return an Error instead if the given range is empty or if it contains an index that is
    /// out of range.
    pub(super) fn remove_nodes(&mut self, range: Range<usize>) -> Result<(), ()> {
        if range.is_empty() || range.end > self.nodes.len() {
            return Err(());
        }

        // Create a new Vec of pointers to the nodes.
        let mut new_vec = Vec::<Shared<Box<dyn AudioGraphNode>>>::with_capacity(
            self.nodes.len() - (range.end - range.start),
        );

        // Clone only the elements not in the range.
        for (i, node) in self.nodes.iter().enumerate() {
            if !range.contains(&i) {
                new_vec.push(Shared::clone(node));
            }
        }

        // Our reference to the old Vec of pointers is dropped here and is replaced with the new one.
        // The old Vec will be fully dropped once the rt thread finishes using it.
        self.nodes = new_vec;

        Ok(())
    }
}
