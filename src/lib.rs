#![no_std]

mod constants {
    pub const NUM_OF_SLABS: usize = 8;
    pub const PAGE_SIZE: usize = 4096;
}

struct Slab {
    block_size: usize,
    free_block_list: usize,
}

enum SlabKind {
    Slab64Bytes,
    Slab128Bytes,
    Slab256Bytes,
    Slab512Bytes,
    Slab1024Bytes,
    Slab2048Bytes,
    Slab4096Bytes,
}

pub struct SlabAllocator {
    slab_64_bytes: Slab,
    slab_128_bytes: Slab,
    slab_256_bytes: Slab,
    slab_512_bytes: Slab,
    slab_1024_bytes: Slab,
    slab_2048_bytes: Slab,
    slab_4096_bytes: Slab,
}

impl SlabAllocator {
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        assert!(
            start_addr % constants::PAGE_SIZE == 0,
            "Start address should be page aligned"
        );

        let slab_size = heap_size / constants::NUM_OF_SLABS;
        SlabAllocator {
            slab_64_bytes: Slab::new(start_addr, slab_size, 64),
            slab_128_bytes: Slab::new(start_addr + slab_size, slab_size, 128),
            slab_256_bytes: Slab::new(start_addr + 2 * slab_size, slab_size, 256),
            slab_512_bytes: Slab::new(start_addr + 3 * slab_size, slab_size, 512),
            slab_1024_bytes: Slab::new(start_addr + 4 * slab_size, slab_size, 1024),
            slab_2048_bytes: Slab::new(start_addr + 5 * slab_size, slab_size, 2048),
            slab_4096_bytes: Slab::new(start_addr + 6 * slab_size, slab_size, 4096),
        }
    }
}
