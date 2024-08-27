//! Implementation of buddy system.
//!
//! ref: [https://github.com/evanw/buddy-malloc](https://github.com/evanw/buddy-malloc)

mod list;

use super::constants;
use list::FreeMemoryBlock;

use alloc::alloc::Layout;
use alloc::rc::Rc;
use core::cell::RefCell;

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
    /// Return smaller size.
    pub fn smaller(self) -> Self {
        match self {
            Self::Byte4K => panic!("Byte4K is min size block"),
            Self::Byte8K => BlockSize::Byte4K,
            Self::Byte16K => BlockSize::Byte8K,
            Self::Byte32K => BlockSize::Byte16K,
            Self::Byte64K => BlockSize::Byte32K,
            Self::Byte128K => BlockSize::Byte64K,
            Self::Byte256K => BlockSize::Byte128K,
            Self::Byte512K => BlockSize::Byte256K,
            Self::Byte1024K => BlockSize::Byte512K,
        }
    }

    /// Return bigger size.
    pub fn bigger(self) -> Self {
        match self {
            Self::Byte4K => BlockSize::Byte8K,
            Self::Byte8K => BlockSize::Byte16K,
            Self::Byte16K => BlockSize::Byte32K,
            Self::Byte32K => BlockSize::Byte64K,
            Self::Byte64K => BlockSize::Byte128K,
            Self::Byte128K => BlockSize::Byte256K,
            Self::Byte256K => BlockSize::Byte512K,
            Self::Byte512K => BlockSize::Byte1024K,
            Self::Byte1024K => panic!("Byte1024K is max size block"),
        }
    }

    /// Return log 2 of self
    pub fn log2(&self) -> usize {
        match self {
            Self::Byte4K => 12,
            Self::Byte8K => 13,
            Self::Byte16K => 14,
            Self::Byte32K => 15,
            Self::Byte64K => 16,
            Self::Byte128K => 17,
            Self::Byte256K => 18,
            Self::Byte512K => 19,
            Self::Byte1024K => 20,
        }
    }

    /// Return `log2_(self) - log2_(Byte4K)`
    pub fn index(&self) -> usize {
        self.log2() - Self::Byte4K.log2()
    }
}

struct BuddyManager {
    /// Base address of entire memory blocks
    base_addr: usize,
    /// Buddy (two child of self) state
    /// - 0: Unused or BothUsed
    /// - 1: Splited (OneUsed)
    ///
    /// It indicate two child state of block, so minimum block does not require this one.
    buddy_state: [u8; (1 << (constants::NUM_OF_BUDDY_SIZE - 1)) / 8],
}

impl BuddyManager {
    pub fn new(base_addr: usize) -> Self {
        BuddyManager {
            base_addr,
            buddy_state: [0u8; (1 << (constants::NUM_OF_BUDDY_SIZE - 1)) / 8],
        }
    }

    fn get_state(&self, index: usize) -> bool {
        (self.buddy_state[index / 8] >> (index % 8)) & 1 == 1
    }

    fn flip_state(&mut self, index: usize) {
        self.buddy_state[index / 8] ^= 1 << (index % 8);
    }

    fn ptr_to_index(&self, block_ptr: *const FreeMemoryBlock) -> usize {
        let block_addr = block_ptr as usize;
        let addr_offset = block_addr - self.base_addr;
        let buddy_index_start = 1 << unsafe { (*block_ptr).size.index() };
        let buddy_index_offset = addr_offset >> unsafe { (*block_ptr).size.log2() };

        buddy_index_start + buddy_index_offset
    }

    pub fn flip_buddy_state(&mut self, block_ptr: *const FreeMemoryBlock) {
        let buddy_index = self.ptr_to_index(block_ptr);
        let parant_buddy_index = (buddy_index - 1) / 2;
        self.flip_state(parant_buddy_index);
    }

    pub fn is_mergeable(&self, block_ptr: *const FreeMemoryBlock) -> bool {
        let buddy_index = self.ptr_to_index(block_ptr);
        let parant_buddy_index = (buddy_index - 1) / 2;
        self.get_state(parant_buddy_index)
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
    _buddy_manager: Rc<RefCell<BuddyManager>>,
}

impl BuddySystem {
    /// Return all empty lists.
    fn new_empty(start_addr: usize) -> Self {
        let buddy_manager = Rc::new(RefCell::new(BuddyManager::new(start_addr)));
        BuddySystem {
            block_4k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte4K,
                Rc::clone(&buddy_manager),
            ),
            block_8k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte8K,
                Rc::clone(&buddy_manager),
            ),
            block_16k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte16K,
                Rc::clone(&buddy_manager),
            ),
            block_32k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte32K,
                Rc::clone(&buddy_manager),
            ),
            block_64k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte64K,
                Rc::clone(&buddy_manager),
            ),
            block_128k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte128K,
                Rc::clone(&buddy_manager),
            ),
            block_256k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte256K,
                Rc::clone(&buddy_manager),
            ),
            block_512k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte512K,
                Rc::clone(&buddy_manager),
            ),
            block_1024k_bytes: list::MemoryBlockList::new_empty(
                BlockSize::Byte1024K,
                Rc::clone(&buddy_manager),
            ),
            _buddy_manager: buddy_manager,
        }
    }

    /// Allocate memory blocks to the largest list of block sizes that can be allocated
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        assert!(start_addr % constants::PAGE_SIZE == 0);
        let current_addr = start_addr;
        let remain_size = heap_size;
        let mut new_lists = Self::new_empty(start_addr);

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

    fn split_request(&mut self, corresponding_block_size: BlockSize) -> *mut u8 {
        assert!(matches!(corresponding_block_size, BlockSize::Byte1024K));
        let bigger_block_size = corresponding_block_size.bigger();
        let bigger_list = match bigger_block_size {
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

        match bigger_list.pop() {
            Some(parent) => {
                let (first_child, second_child) = parent.split();
                let (first_child, second_child) = (
                    first_child as *mut FreeMemoryBlock,
                    second_child as *mut FreeMemoryBlock,
                );
                unsafe {
                    *first_child = FreeMemoryBlock::new(corresponding_block_size);
                    *second_child = FreeMemoryBlock::new(corresponding_block_size);

                    let corresponding_list = match corresponding_block_size {
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
                    corresponding_list.append(&mut *first_child);
                }

                first_child as *mut u8
            }
            None => self.split_request(bigger_block_size),
        }
    }

    /// Allocates a new memory block.
    pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
        let corresponding_block_size = Self::get_memory_block_size(&layout);
        let corresponding_block_list = match corresponding_block_size {
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

        match corresponding_block_list.pop() {
            Some(refer) => refer as *mut FreeMemoryBlock as *mut u8,
            None => self.split_request(corresponding_block_size),
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
        let mut block_ptr = ptr as *mut FreeMemoryBlock;
        while let Some(merged) = corresponding_list.append(&mut *block_ptr) {
            block_ptr = merged;
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
            0x100000..0x200000 => BlockSize::Byte1024K,
            _ => panic!("requested size is too large"),
        }
    }
}
