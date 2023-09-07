#![no_std]

mod constants {
    const NUM_OF_SLABS: usize = 8;
}

struct Slab {
    block_size: usize,
    free_block_list: usize,
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
