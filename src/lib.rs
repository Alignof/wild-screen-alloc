#![no_std]

mod slab;

use slab::SlabCache;

mod constants {
    pub const NUM_OF_SLABS: usize = 8;
    pub const PAGE_SIZE: usize = 4096;
}

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
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        assert!(
            start_addr % constants::PAGE_SIZE == 0,
            "Start address should be page aligned"
        );

        let alloc_size = heap_size / constants::NUM_OF_SLABS;
        SlabAllocator {
            slab_64_bytes: SlabCache::new(start_addr, alloc_size, 64),
            slab_128_bytes: SlabCache::new(start_addr + alloc_size, alloc_size, 128),
            slab_256_bytes: SlabCache::new(start_addr + 2 * alloc_size, alloc_size, 256),
            slab_512_bytes: SlabCache::new(start_addr + 3 * alloc_size, alloc_size, 512),
            slab_1024_bytes: SlabCache::new(start_addr + 4 * alloc_size, alloc_size, 1024),
            slab_2048_bytes: SlabCache::new(start_addr + 5 * alloc_size, alloc_size, 2048),
            slab_4096_bytes: SlabCache::new(start_addr + 6 * alloc_size, alloc_size, 4096),
        }
    }
}
