//! Implementation of buddy system.
//!
//! ref: [https://github.com/evanw/buddy-malloc](https://github.com/evanw/buddy-malloc)

mod list;

use super::constants;

/// Block size that is managed by buddy system.
#[derive(Copy, Clone)]
pub enum BlockSize {
    Byte4K = 4 * 1024, // = PAGE_SIZE
    Byte8K = 8 * 1024,
    Byte16K = 16 * 1024,
    Byte32K = 32 * 1024,
    Byte64K = 64 * 1024,
    Byte128K = 128 * 1024,
    Byte256K = 256 * 1024,
    Byte512K = 512 * 1024,
    Byte1024K = 1024 * 1024,
}

impl BlockSize {
    fn header_size(&self) -> usize {
        match self {
            Self::Byte4K => size_of::<MemoryBlockHeader>() * 1,
            Self::Byte8K => size_of::<MemoryBlockHeader>() * 2,
            Self::Byte16K => size_of::<MemoryBlockHeader>() * 4,
            Self::Byte32K => size_of::<MemoryBlockHeader>() * 8,
            Self::Byte64K => size_of::<MemoryBlockHeader>() * 16,
            Self::Byte128K => size_of::<MemoryBlockHeader>() * 32,
            Self::Byte256K => size_of::<MemoryBlockHeader>() * 64,
            Self::Byte512K => size_of::<MemoryBlockHeader>() * 128,
            Self::Byte1024K => size_of::<MemoryBlockHeader>() * 256,
        }
    }

    pub fn size_with_header(&self) -> usize {
        let block_size = self.clone() as usize;
        self.header_size() + block_size
    }
}

/// Header of memory block
struct MemoryBlockHeader {
    /// Is memory block used?
    is_used: bool,
    /// Memory block size.
    size: BlockSize,
    /// Next empty node of linked list.
    next: Option<&'static mut MemoryBlockHeader>,
    /// Parent address
    /// ```ignore
    /// match self.parent {
    ///     Some(addr) => {
    ///         if addr == &self {
    ///             // rhs
    ///         } else {
    ///             // lhs
    ///         }
    ///     }
    ///     None => // root
    /// }
    /// ```
    parent: Option<&'static mut MemoryBlockHeader>,
}

pub struct BuddySystem {
    block_4k_bytes: list::MemoryBlockList,
}
