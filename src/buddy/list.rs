//! Implementation for linked list for buddy system.

use super::MemoryBlockHeader;

/// Linked list of memory block
pub struct MemoryBlockList {
    pub head: Option<&'static mut MemoryBlockHeader>,
}

impl MemoryBlockList {
    /// Return with empty head.
    pub fn new_empty() -> Self {
        MemoryBlockList { head: None }
    }

    /// Append new memory block
    fn append(&mut self, mem_block: &'static mut MemoryBlockHeader) {
        mem_block.next = self.head.take();
        self.head = Some(mem_block);
    }

    /// Pop free memory block
    fn pop(&mut self) -> Option<&'static mut MemoryBlockHeader> {
        self.head.take().map(|header| {
            self.head = header.next.take();
            header
        })
    }
}
