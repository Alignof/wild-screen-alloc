//! Implementation for linked list of Slab

use super::{FreeObject, ObjectSize, Slab};
use crate::buddy;

use alloc::sync::Arc;
use core::cell::OnceCell;
use spin::Mutex;

/// Linked list of Slab
pub struct List {
    /// List length.
    len: usize,
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
            head: unsafe { Some(Slab::new(obj_size, new_page_addr)) },
        }
    }

    /// Return with empty list.
    pub fn new_empty() -> Self {
        List { len: 0, head: None }
    }
}

pub struct EmptyList(List);

impl EmptyList {
    pub fn new(
        obj_size: ObjectSize,
        default_node_num: usize,
        page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        EmptyList(List::new(obj_size, default_node_num, page_allocator))
    }

    /// Return with empty list.
    pub fn new_empty() -> Self {
        EmptyList(List::new_empty())
    }

    /// Create new node and append to list.
    pub fn append_new_node(
        &mut self,
        obj_size: ObjectSize,
        page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) {
        let new_page_addr = page_allocator.lock().get_mut().unwrap().page_allocate() as *mut Slab;
        let new_node = unsafe { Slab::new(obj_size, new_page_addr) };
        new_node.next = self.0.head.take();
        self.0.len += 1;
        self.0.head = Some(new_node);
    }
}

pub struct PartialList(List);

impl PartialList {
    /// Return with empty list.
    pub fn new_empty() -> Self {
        PartialList(List::new_empty())
    }

    /// Pop free object from list of head
    pub fn pop_object(&mut self) -> Option<&'static mut FreeObject> {
        self.0.head.as_mut().expect("Slab list is empty").pop()
    }

    /// Push deallocated object to corresponding `Slab`
    pub fn push_object(&mut self, obj: &'static mut FreeObject) {
        self.0.head.as_mut().unwrap().push(obj);
    }
}
