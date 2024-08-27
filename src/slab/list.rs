//! Implementation for linked list of Slab

use super::{FreeObject, ObjectSize, Slab, SlabKind};
use crate::buddy;

use alloc::sync::Arc;
use core::cell::OnceCell;
use spin::Mutex;

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
    /// Slab
    slab: Slab,
    /// Next node pointer
    next: Option<&'static mut Self>,
}

impl Node {
    /// Map `Node` structure to allocated memory block.
    fn new(obj_size: ObjectSize, allocated_page_ptr: *mut Self) -> &'static mut Self {
        unsafe {
            *allocated_page_ptr = Node {
                next: None,
                slab: Slab::new(obj_size, allocated_page_ptr as usize + size_of::<Node>()),
            };

            &mut *allocated_page_ptr
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
    /// List length.
    len: usize,
    /// Reference of `BuddySystem` for allocating Page
    page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    /// head of `Node` linked list.
    pub head: Option<&'static mut Node>,
}

impl List {
    /// Return with initialize Slab.
    pub fn new(
        obj_size: ObjectSize,
        default_node_num: usize,
        page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        let new_page = page_allocator.lock().get_mut().unwrap().page_allocate() as *mut Node;
        List {
            len: default_node_num,
            page_allocator,
            head: Some(Node::new(obj_size, new_page)),
        }
    }

    /// Return with empty head.
    pub fn new_empty(page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>) -> Self {
        List {
            len: 0,
            page_allocator,
            head: None,
        }
    }
}
