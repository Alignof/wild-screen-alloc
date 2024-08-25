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
    obj_size: ObjectSize,
    head: Option<&'static mut Node>,
}

impl List {
    /// Return with initialize Slab.
    pub fn new(obj_size: ObjectSize) -> Self {
        List {
            len: 1,
            obj_size,
            head: Some(&'static mut Node::new(obj_size)),
        }
    }

    /// Return with empty head.
    pub fn new_empty(obj_size: ObjectSize) -> Self {
        List {
            len: 0,
            obj_size,
            head: None,
        }
    }

    /// Push new free object.
    pub fn push(&mut self, node: &'static mut Node) {
        node.next = self.head.take();
        self.len += 1;
        self.head = Some(node);
    }

    /// Pop free object.
    pub fn pop(&mut self) -> Option<&'static mut Node> {
        self.head.take().map(|node| {
            self.head = node.next.take();
            self.len -= 1;
            node
        })
    }
}
