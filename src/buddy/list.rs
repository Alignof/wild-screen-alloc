//! Implementation for linked list for buddy system.

use super::{BlockSize, BuddyManager};

use alloc::rc::Rc;
use core::cell::RefCell;

/// Node of `MemoryBlockList`
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FreeMemoryBlock {
    /// Memory block size.
    pub size: BlockSize,
    /// Next empty node of linked list.
    next: Option<&'static mut Self>,
}

impl FreeMemoryBlock {
    /// Return new `FreeMemoryBlock`
    pub fn new(size: BlockSize) -> Self {
        FreeMemoryBlock { size, next: None }
    }

    /// Split memory block into two smaller block reference
    /// Block isn't initialized here.
    pub fn split(&mut self) -> (&'static mut Self, &'static mut Self) {
        let self_ptr = self as *mut Self;
        assert!(self_ptr as usize % self.size.smaller() as usize == 0);
        debug_assert!(self.next.is_none());

        let new_size = self.size.smaller();
        let first_child = self_ptr;
        let second_child = unsafe { first_child.byte_add(self.size.smaller() as usize) };
        let (first_child, second_child) = unsafe { (&mut *first_child, &mut *second_child) };
        first_child.size = new_size;
        second_child.size = new_size;

        (first_child, second_child)
    }

    /// Is first half child
    ///
    /// This method used to return address of parant block
    fn is_first_half(&self) -> bool {
        let self_addr = std::ptr::from_ref::<Self>(self) as usize;
        self_addr % self.size.bigger() as usize == 0
    }

    /// Get buddy
    ///
    /// Address is calculated by self address.
    fn get_buddy(&mut self) -> &'static mut Self {
        let self_addr = self as *mut Self;
        unsafe {
            if self.is_first_half() {
                &mut *(self_addr.byte_add(self.size as usize))
            } else {
                &mut *(self_addr.byte_sub(self.size as usize))
            }
        }
    }

    /// Try merge memory block to double
    pub fn try_merge(
        &mut self,
        buddy_manager: &Rc<RefCell<BuddyManager>>,
    ) -> Option<&'static mut Self> {
        // Mex size block can not merge
        if matches!(self.size, BlockSize::Byte1024K) {
            return None;
        }

        let mut buddy_manager = buddy_manager.borrow_mut();
        if buddy_manager.is_mergeable(self) {
            // change buddy state splited to unused
            buddy_manager.flip_buddy_state(self);

            // return pointer of head of one
            if self.is_first_half() {
                self.size = self.size.bigger();
                unsafe { Some(&mut *std::ptr::from_mut::<Self>(self)) }
            } else {
                let buddy = self.get_buddy();
                buddy.size = buddy.size.bigger();
                Some(buddy)
            }
        } else {
            None
        }
    }
}

/// Linked list of memory block
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct MemoryBlockList {
    /// Memory block size.
    block_size: BlockSize,
    /// Reference of `BuddyManager`.
    buddy_manager: Rc<RefCell<BuddyManager>>,
    /// Head block of the linked list.
    pub head: Option<&'static mut FreeMemoryBlock>,
}

impl MemoryBlockList {
    /// Return with empty head.
    pub fn new_empty(block_size: BlockSize, buddy_manager: Rc<RefCell<BuddyManager>>) -> Self {
        MemoryBlockList {
            block_size,
            buddy_manager,
            head: None,
        }
    }

    /// Append memory block greedily from raw pointer.
    /// It used for initialize this.
    pub fn initialize(&mut self, start_addr: usize) {
        assert!(start_addr % self.block_size as usize == 0);
        let new_header_ptr = start_addr as *mut FreeMemoryBlock;
        unsafe {
            *new_header_ptr = FreeMemoryBlock::new(self.block_size);
            self.append(&mut *new_header_ptr);
        }
    }

    /// Append new memory block
    pub fn append(
        &mut self,
        mem_block: &'static mut FreeMemoryBlock,
    ) -> Option<&'static mut FreeMemoryBlock> {
        mem_block.next = self.head.take();
        let merge_result = mem_block.try_merge(&self.buddy_manager);
        if merge_result.is_none() {
            self.head = Some(mem_block);
        }

        merge_result
    }

    /// Pop free memory block
    #[allow(clippy::manual_inspect)]
    pub fn pop(&mut self) -> Option<&'static mut FreeMemoryBlock> {
        self.head.take().map(|header| {
            self.head = header.next.take();
            header
        })
    }
}
