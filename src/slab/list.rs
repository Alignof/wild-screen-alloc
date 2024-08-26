//! Implementation for linked list of Slab

use super::{ObjectSize, Slab, SlabKind};

/// Node of `List`
struct Node {
    next: Option<&'static mut Self>,
    slab: Slab,
}

impl Node {
    fn new(obj_size: ObjectSize) -> Self {
        Node {
            next: None,
            slab: unsafe { Slab::new(obj_size) },
        }
    }

    fn kind(&self) -> &SlabKind {
        &self.slab.kind
    }
}

/// Linked list of Slab
pub struct List {
    len: usize,
    pub head: Option<&'static mut Node>,
}

impl List {
    /// Return with initialize Slab.
    pub fn new(obj_size: ObjectSize, default_node_num: usize) -> Self {
        List {
            len: default_node_num,
            head: Some(Node::new(obj_size)),
        }
    }

    /// Return with empty head.
    pub fn new_empty() -> Self {
        List { len: 0, head: None }
    }
}
