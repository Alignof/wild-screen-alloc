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

    /// Push new free object.
    fn push_slab(&mut self, slab: &'static mut Slab) {
        slab.next = self.head.take();
        self.len += 1;
        self.head = Some(slab);
    }

    /// Pop free object.
    fn pop_slab(&mut self) -> Option<&'static mut Slab> {
        self.head.take().map(|slab| {
            self.head = slab.next.take();
            self.len -= 1;
            slab
        })
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

    /// Push new free object.
    pub fn push_slab(&mut self, slab: &'static mut Slab) {
        self.0.push_slab(slab);
    }

    /// Pop free object.
    pub fn pop_slab(&mut self) -> Option<&'static mut Slab> {
        self.0.pop_slab()
    }
}

pub struct PartialList(pub List);

impl PartialList {
    /// Return with empty list.
    pub fn new_empty() -> Self {
        PartialList(List::new_empty())
    }

    /// Push new free object.
    pub fn push_slab(&mut self, slab: &'static mut Slab) {
        self.0.push_slab(slab);
    }

    /// Pop free object.
    pub fn pop_slab(&mut self) -> Option<&'static mut Slab> {
        self.0.pop_slab()
    }

    /// Return pointer of list head.
    pub fn head_ptr(&mut self) -> Option<*mut Slab> {
        self.0.head.as_mut().map(|slab| *slab as *mut Slab)
    }

    /// Search slab that contains given free object.
    pub fn corresponding_slab_ptr(&mut self, obj_ptr: *const FreeObject) -> Option<*mut Slab> {
        let mut next_slab = self.0.head.take();
        while let Some(slab) = next_slab {
            if slab.is_contain(obj_ptr) {
                return Some(slab as *mut Slab);
            } else {
                next_slab = slab.next.take();
            }
        }

        None
    }
}

pub struct FullList(List);

impl FullList {
    /// Return with empty list.
    pub fn new_empty() -> Self {
        FullList(List::new_empty())
    }

    /// Push new free object.
    pub fn push_slab(&mut self, slab: &'static mut Slab) {
        self.0.push_slab(slab);
    }

    /// Pop free object.
    pub fn pop_slab(&mut self) -> Option<&'static mut Slab> {
        self.0.pop_slab()
    }

    /// Search slab that contains given free object.
    pub fn corresponding_slab_ptr(&mut self, obj_ptr: *const FreeObject) -> Option<*mut Slab> {
        let mut next_slab = self.0.head.take();
        while let Some(slab) = next_slab {
            if slab.is_contain(obj_ptr) {
                return Some(slab as *mut Slab);
            } else {
                next_slab = slab.next.take();
            }
        }

        None
    }
}
