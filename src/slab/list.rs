//! Implementation for linked list of Slab

use super::{ObjectSize, Slab};
use crate::buddy;

use alloc::sync::Arc;
use core::cell::OnceCell;
use spin::Mutex;

/// Linked list of Slab
pub struct List {
    /// List length.
    len: usize,
    /// Reference of `BuddySystem` for allocating Page
    page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    /// head of `Slab` linked list.
    pub head: Option<&'static mut Slab>,
}

impl List {
    /// Return with initialize Slab.
    pub fn new(
        obj_size: ObjectSize,
        default_node_num: usize,
        page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        let new_page_addr = page_allocator.lock().get_mut().unwrap().page_allocate() as *mut Slab;
        List {
            len: default_node_num,
            page_allocator,
            head: unsafe { Some(Slab::new(obj_size, new_page_addr)) },
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

    /// Create new node and append to list.
    pub fn append_new_node(&mut self, obj_size: ObjectSize) {
        let new_page_addr = self
            .page_allocator
            .lock()
            .get_mut()
            .unwrap()
            .page_allocate() as *mut Slab;
        let new_node = unsafe { Slab::new(obj_size, new_page_addr) };
        new_node.next = self.head.take();
        self.len += 1;
        self.head = Some(new_node);
    }
}
