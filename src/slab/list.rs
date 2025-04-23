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
    /// Return initialized Slab.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn new(
        obj_size: ObjectSize,
        default_node_num: usize,
        page_allocator: &Rc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        let new_page_addr = page_allocator
            .lock()
            .get_mut()
            .unwrap()
            .page_allocate()
            .cast::<Slab>();
        List {
            len: default_node_num,
            head: unsafe { Some(Slab::new(obj_size, new_page_addr)) },
        }
    }

    /// Return empty list.
    pub fn new_empty() -> Self {
        List { len: 0, head: None }
    }

    /// Push new `Slab`
    fn push_slab(&mut self, slab: &'static mut Slab) {
        slab.next = self.head.take();
        self.len += 1;
        self.head = Some(slab);
    }

    /// Pop `Slab` from the list.
    ///
    /// If the list is empty, new Slab is allocated from new page.
    ///
    /// # Safety
    ///
    /// If a new page needs to be allocated, this function relies on the `page_allocator`
    /// returning a valid, page-aligned pointer to a memory region of at least `PAGE_SIZE`.
    /// The allocated page is then initialized via `Slab::new`, which itself is unsafe
    /// and requires the pointer to be valid.
    #[allow(clippy::manual_inspect)]
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
        page_allocator: &Rc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        EmptyList(List::new(obj_size, default_node_num, page_allocator))
    }

    /// Create new empty list.
    #[allow(dead_code)]
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
    #[allow(clippy::cast_ptr_alignment)]
    pub fn pop_slab(
        &mut self,
        obj_size: ObjectSize,
        page_allocator: &Rc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> &'static mut Slab {
        self.0.pop_slab().unwrap_or_else(|| {
            let new_page_addr = page_allocator
                .lock()
                .get_mut()
                .unwrap()
                .page_allocate()
                .cast::<Slab>();
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
        self.0
            .head
            .as_mut()
            .map(|slab| std::ptr::from_mut::<Slab>(*slab))
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
            }

            next_slab = slab.next.take();
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
    #[allow(dead_code)]
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
            }

            next_slab = slab.next.take();
        }

        None
    }
}
