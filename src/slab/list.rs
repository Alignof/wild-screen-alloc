//! Implementation for linked list of Slab

use super::{FreeObject, ObjectSize, Slab, SlabKind};

/// Node of `List`
///
/// It's mapped as page and contains `Slab`.
///
/// # Memory layout
/// ```ignore
/// ┌──────────────────────────────────────────────────────────────────────────┐
/// │                                                                          │
/// │                           size_of::<Node>()                              │
/// │   ◄──────────────────────────────────────────────────────────────────►   │
/// │                                    size_of::<Slab>()                     │
/// │  0              ◄────────────────────────────────────────────────────►   │
/// │  ┌─────────────┬──────────────────────────────────────────────────────┐  │
/// │  │  Node.next  │                    Node.slab                        ─┼──┘
/// │  ├─────────────┼─────────────┬─────────────┬────────────┬─────────────┤   
/// └───► free_obj   │  free_obj   │  free_obj   │  free_obj  │  free_obj   │   
///    ├─────────────┼─────────────┼─────────────┼────────────┼─────────────┤   
///    │  free_obj   │  free_obj   │  free_obj   │  free_obj  │  free_obj   │   
///    └─────────────┴─────────────┴─────────────┴────────────┴─────────────┘   
///                                                                       4096
/// ```
#[repr(C)]
pub struct Node {
    /// Next node pointer
    next: Option<&'static mut Self>,
    /// Slab
    slab: Slab,
}

impl Node {
    /// Map `Node` structure to allocated memory block.
    fn new(obj_size: ObjectSize) -> &'static mut Self {
        // TODO: allocate it by buddy system.
        let dummy_page_ptr = [0u8; 4096].as_mut_ptr() as *mut Node;

        unsafe {
            *dummy_page_ptr = Node {
                next: None,
                slab: Slab::new(obj_size, dummy_page_ptr as usize + size_of::<Node>()),
            };

            &mut *dummy_page_ptr
        }
    }

    /// Return `SlabKind`
    fn kind(&self) -> &SlabKind {
        &self.slab.kind
    }

    /// Push new free object.
    pub fn push(&mut self, slab: &'static mut FreeObject) {
        self.slab.push(slab)
    }

    /// Pop free object.
    pub fn pop(&mut self) -> Option<&'static mut FreeObject> {
        self.slab.pop()
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
