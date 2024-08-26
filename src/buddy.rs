//! Implementation of buddy system.
//!
//! ref: [https://github.com/evanw/buddy-malloc](https://github.com/evanw/buddy-malloc)

mod list;

use super::constants;
use alloc::alloc::Layout;

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

enum MemoryBlockType {
    /// First half of parent.
    /// It will be parent.
    FirstChild,
    /// Second half of parent.
    /// It will merge into `FirstChild` when doubling.
    SecondChild,
    /// No parent. (root)
    Orphan,
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
    kind: MemoryBlockType,
}

impl MemoryBlockHeader {
    pub fn new(size: BlockSize) -> Self {
        MemoryBlockHeader {
            is_used: false,
            size,
            next: None,
            kind: MemoryBlockType::Orphan,
        }
    }
}

pub struct BuddySystem {
    block_4k_bytes: list::MemoryBlockList,
    block_8k_bytes: list::MemoryBlockList,
    block_16k_bytes: list::MemoryBlockList,
    block_32k_bytes: list::MemoryBlockList,
    block_64k_bytes: list::MemoryBlockList,
    block_128k_bytes: list::MemoryBlockList,
    block_256k_bytes: list::MemoryBlockList,
    block_512k_bytes: list::MemoryBlockList,
    block_1024k_bytes: list::MemoryBlockList,
}

impl BuddySystem {
    /// Return all empty lists.
    fn new_empty() -> Self {
        BuddySystem {
            block_4k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte4K),
            block_8k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte8K),
            block_16k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte16K),
            block_32k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte32K),
            block_64k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte64K),
            block_128k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte128K),
            block_256k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte256K),
            block_512k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte512K),
            block_1024k_bytes: list::MemoryBlockList::new_empty(BlockSize::Byte1024K),
        }
    }

    /// Allocate memory blocks to the largest list of block sizes that can be allocated
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        assert!(start_addr % constants::PAGE_SIZE == 0);
        let current_addr = start_addr;
        let remain_size = heap_size;
        let mut new_lists = Self::new_empty();

        let (current_addr, remain_size) = new_lists
            .block_1024k_bytes
            .initialize_greedily(current_addr, remain_size);
        let (current_addr, remain_size) = new_lists
            .block_512k_bytes
            .initialize_greedily(current_addr, remain_size);
        let (current_addr, remain_size) = new_lists
            .block_256k_bytes
            .initialize_greedily(current_addr, remain_size);
        let (current_addr, remain_size) = new_lists
            .block_128k_bytes
            .initialize_greedily(current_addr, remain_size);
        let (current_addr, remain_size) = new_lists
            .block_64k_bytes
            .initialize_greedily(current_addr, remain_size);
        let (current_addr, remain_size) = new_lists
            .block_32k_bytes
            .initialize_greedily(current_addr, remain_size);
        let (current_addr, remain_size) = new_lists
            .block_16k_bytes
            .initialize_greedily(current_addr, remain_size);
        let (current_addr, remain_size) = new_lists
            .block_8k_bytes
            .initialize_greedily(current_addr, remain_size);
        new_lists
            .block_4k_bytes
            .initialize_greedily(current_addr, remain_size);

        new_lists
    }

    /// Allocates a new memory block.
    pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
        let corresponding_block_size = Self::get_memory_block_size(&layout);
        let corresponding_list = match corresponding_block_size {
            BlockSize::Byte4K => self.block_4k_bytes,
            BlockSize::Byte8K => self.block_8k_bytes,
            BlockSize::Byte16K => self.block_16k_bytes,
            BlockSize::Byte32K => self.block_32k_bytes,
            BlockSize::Byte64K => self.block_64k_bytes,
            BlockSize::Byte128K => self.block_128k_bytes,
            BlockSize::Byte256K => self.block_256k_bytes,
            BlockSize::Byte512K => self.block_512k_bytes,
            BlockSize::Byte1024K => self.block_1024k_bytes,
        };

        match corresponding_list.pop() {
            Some(ptr) => unsafe {
                (ptr as *mut MemoryBlockHeader).byte_add(size_of::<MemoryBlockHeader>()) as *mut u8
            },
            None => self.split_request(corrensponding_block_size),
        }
    }

    /// Deallocate(free) object.
    /// # Safety
    /// Given pointer must be valid.
    ///
    /// # Panics
    /// If given ptr is null, it will panic.
    pub unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        let corresponding_block_size = Self::get_memory_block_size(&layout);
        let mut corresponding_list = match corresponding_block_size {
            BlockSize::Byte4K => &mut self.block_4k_bytes,
            BlockSize::Byte8K => &mut self.block_8k_bytes,
            BlockSize::Byte16K => &mut self.block_16k_bytes,
            BlockSize::Byte32K => &mut self.block_32k_bytes,
            BlockSize::Byte64K => &mut self.block_64k_bytes,
            BlockSize::Byte128K => &mut self.block_128k_bytes,
            BlockSize::Byte256K => &mut self.block_256k_bytes,
            BlockSize::Byte512K => &mut self.block_512k_bytes,
            BlockSize::Byte1024K => &mut self.block_1024k_bytes,
        };

        // merge child block and move doubled block to corresponding list
        let body_addr = ptr as usize;
        let mut header_ptr = (body_addr - size_of::<MemoryBlockHeader>()) as *mut MemoryBlockHeader;
        while let Some(merged) = corresponding_list.append(&mut *header_ptr) {
            header_ptr = merged;
            corresponding_list = match corresponding_block_size {
                BlockSize::Byte4K => &mut self.block_8k_bytes,
                BlockSize::Byte8K => &mut self.block_16k_bytes,
                BlockSize::Byte16K => &mut self.block_32k_bytes,
                BlockSize::Byte32K => &mut self.block_64k_bytes,
                BlockSize::Byte64K => &mut self.block_128k_bytes,
                BlockSize::Byte128K => &mut self.block_256k_bytes,
                BlockSize::Byte256K => &mut self.block_512k_bytes,
                BlockSize::Byte512K => &mut self.block_1024k_bytes,
                BlockSize::Byte1024K => unreachable!(),
            }
        }
    }

    fn get_memory_block_size(layout: &Layout) -> BlockSize {
        match layout.size() {
            0x1000..0x2000 => BlockSize::Byte4K,
            0x2000..0x4000 => BlockSize::Byte8K,
            0x4000..0x8000 => BlockSize::Byte16K,
            0x8000..0x10000 => BlockSize::Byte32K,
            0x10000..0x20000 => BlockSize::Byte64K,
            0x20000..0x40000 => BlockSize::Byte128K,
            0x40000..0x80000 => BlockSize::Byte256K,
            0x80000..0x100000 => BlockSize::Byte512K,
            0x100000..usize::MAX => BlockSize::Byte1024K,
            _ => unreachable!(),
        }
    }
}
