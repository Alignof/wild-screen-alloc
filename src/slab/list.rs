//! Implementation for linked list of Slab

use super::{FreeObject, ObjectSize, Slab};
use crate::buddy;

use alloc::rc::Rc;
use core::cell::OnceCell;
use spin::Mutex;

/// Linked list of Slab
pub struct List {
    /// Length of the list.
    len: usize,
    /// Head of `Slab` linked list.
    pub head: Option<&'static mut Slab>,
}

impl List {
    /// Return with initialized Slab.
    pub fn new(
        obj_size: ObjectSize,
        default_node_num: usize,
        page_allocator: Rc<Mutex<OnceCell<buddy::BuddySystem>>>,
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

    /// Push new `Slab`
    fn push_slab(&mut self, slab: &'static mut Slab) {
        slab.next = self.head.take();
        self.len += 1;
        self.head = Some(slab);
    }

    /// Pop `Slab`.
    fn pop_slab(&mut self) -> Option<&'static mut Slab> {
        self.head.take().map(|slab| {
            self.head = slab.next.take();
            self.len -= 1;
            slab
        })
    }
}

/// List of empty slabs.
pub struct EmptyList(List);

impl EmptyList {
    /// Create new `EmptyList`.
    pub fn new(
        obj_size: ObjectSize,
        default_node_num: usize,
        page_allocator: Rc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        EmptyList(List::new(obj_size, default_node_num, page_allocator))
    }

    /// Create new empty list.
    pub fn new_empty() -> Self {
        EmptyList(List::new_empty())
    }

    /// Push new `Slab` to the list.
    pub fn push_slab(&mut self, slab: &'static mut Slab) {
        self.0.push_slab(slab);
    }

    /// Pop `Slab` from the list.
    ///
    /// If list is empty, new Slab allocate from new page.
    pub fn pop_slab(
        &mut self,
        obj_size: ObjectSize,
        page_allocator: Rc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> &'static mut Slab {
        self.0.pop_slab().unwrap_or_else(|| {
            let new_page_addr =
                page_allocator.lock().get_mut().unwrap().page_allocate() as *mut Slab;
            unsafe { Slab::new(obj_size, new_page_addr) }
        })
    }
}

/// List of partialy used slab.
pub struct PartialList(pub List);

impl PartialList {
    /// Create an empty list.
    pub fn new_empty() -> Self {
        PartialList(List::new_empty())
    }

    /// Push new `Slab` to the list.
    pub fn push_slab(&mut self, slab: &'static mut Slab) {
        self.0.push_slab(slab);
    }

    /// Pop `Slab` from the list.
    pub fn pop_slab(&mut self) -> Option<&'static mut Slab> {
        self.0.pop_slab()
    }

    /// Return the head pointer of the list.
    pub fn peek(&mut self) -> Option<*mut Slab> {
        self.0.head.as_mut().map(|slab| *slab as *mut Slab)
    }

    /// Search and pop slab that contains given free object.
    pub fn pop_corresponding_slab(
        &mut self,
        obj_ptr: *const FreeObject,
    ) -> Option<&'static mut Slab> {
        let mut next_slab = self.0.head.take();
        while let Some(slab) = next_slab {
            if slab.is_contain(obj_ptr) {
                return Some(slab);
            } else {
                next_slab = slab.next.take();
            }
        }

        None
    }
}

/// List of fully used slab.
pub struct FullList(List);

impl FullList {
    /// Create new empty list.
    pub fn new_empty() -> Self {
        FullList(List::new_empty())
    }

    /// Push new `Slab` to the list.
    pub fn push_slab(&mut self, slab: &'static mut Slab) {
        self.0.push_slab(slab);
    }

    /// Pop `Slab` from the list.
    pub fn pop_slab(&mut self) -> Option<&'static mut Slab> {
        self.0.pop_slab()
    }

    /// Search and pop slab that contains given free object.
    pub fn pop_corresponding_slab(
        &mut self,
        obj_ptr: *const FreeObject,
    ) -> Option<&'static mut Slab> {
        let mut next_slab = self.0.head.take();
        while let Some(slab) = next_slab {
            if slab.is_contain(obj_ptr) {
                return Some(slab);
            } else {
                next_slab = slab.next.take();
            }
        }

        None
    }
}
