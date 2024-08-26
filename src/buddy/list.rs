//! Implementation for linked list for buddy system.

use super::{BlockSize, MemoryBlockHeader};

/// Linked list of memory block
pub struct MemoryBlockList {
    block_size: BlockSize,
    pub head: Option<&'static mut MemoryBlockHeader>,
}

impl MemoryBlockList {
    /// Return with empty head.
    pub fn new_empty(block_size: BlockSize) -> Self {
        MemoryBlockList {
            block_size,
            head: None,
        }
    }

    /// Append memory block greedily from raw pointer.
    /// It used for initialize this.
    pub fn initialize_greedily(
        &mut self,
        mut current_addr: usize,
        mut remain_size: usize,
    ) -> (usize, usize) {
        while remain_size < self.block_size as usize {
            let new_header_ptr = current_addr as *mut MemoryBlockHeader;
            unsafe {
                *new_header_ptr = MemoryBlockHeader::new(self.block_size);
                self.append(&mut *new_header_ptr);
            }

            current_addr += self.block_size.size_with_header();
            remain_size -= self.block_size.size_with_header();
        }

        (current_addr, remain_size)
    }

    /// Append new memory block
    pub fn append(&mut self, mem_block: &'static mut MemoryBlockHeader) {
        mem_block.next = self.head.take();
        self.head = Some(mem_block);
    }

    /// Pop free memory block
    pub fn pop(&mut self) -> Option<&'static mut MemoryBlockHeader> {
        self.head.take().map(|header| {
            self.head = header.next.take();
            header
        })
    }
}
