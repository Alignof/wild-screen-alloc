#![no_std]

mod slab;

use slab::{SlabCache, SlabSize};

/// Constants.
mod constants {
    /// Number of slab allocator size.
    pub const NUM_OF_SLABS: usize = 8;
    /// Page size.
    pub const PAGE_SIZE: usize = 4096;
}

/// Slab allocator that provide global allocator.
pub struct SlabAllocator {
    slab_64_bytes: SlabCache,
    slab_128_bytes: SlabCache,
    slab_256_bytes: SlabCache,
    slab_512_bytes: SlabCache,
    slab_1024_bytes: SlabCache,
    slab_2048_bytes: SlabCache,
    slab_4096_bytes: SlabCache,
}

impl SlabAllocator {
    /// Return new SlabAllocator.
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        assert!(
            start_addr % constants::PAGE_SIZE == 0,
            "Start address should be page aligned"
        );

        let slab_allocated_size = heap_size / constants::NUM_OF_SLABS;
        SlabAllocator {
            slab_64_bytes: SlabCache::new(start_addr, slab_allocated_size, SlabSize::Slab64Bytes),
            slab_128_bytes: SlabCache::new(
                start_addr + slab_allocated_size,
                slab_allocated_size,
                SlabSize::Slab128Bytes,
            ),
            slab_256_bytes: SlabCache::new(
                start_addr + 2 * slab_allocated_size,
                slab_allocated_size,
                SlabSize::Slab256Bytes,
            ),
            slab_512_bytes: SlabCache::new(
                start_addr + 3 * slab_allocated_size,
                slab_allocated_size,
                SlabSize::Slab512Bytes,
            ),
            slab_1024_bytes: SlabCache::new(
                start_addr + 4 * slab_allocated_size,
                slab_allocated_size,
                SlabSize::Slab1024Bytes,
            ),
            slab_2048_bytes: SlabCache::new(
                start_addr + 5 * slab_allocated_size,
                slab_allocated_size,
                SlabSize::Slab2048Bytes,
            ),
            slab_4096_bytes: SlabCache::new(
                start_addr + 6 * slab_allocated_size,
                slab_allocated_size,
                SlabSize::Slab4096Bytes,
            ),
        }
    }
}

#[cfg(test)]
mod alloc_tests {
    use crate::{constants, SlabAllocator};

    const HEAP_SIZE: usize = 8 * constants::PAGE_SIZE;

    #[repr(align(4096))]
    struct DummyHeap {
        heap_space: [u8; HEAP_SIZE],
    }

    #[test]
    fn alloc_heap() {
        let dummy_heap = DummyHeap {
            heap_space: [0_u8; HEAP_SIZE],
        };

        unsafe {
            SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
        }
    }
}
