#![no_std]

extern crate alloc;

mod slab;

use alloc::alloc::Layout;
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

    /// Allocates a new object.
    pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
        match Self::get_slab_size(&layout) {
            slab::SlabSize::Slab64Bytes => self.slab_64_bytes.allocate(layout),
            slab::SlabSize::Slab128Bytes => self.slab_64_bytes.allocate(layout),
            slab::SlabSize::Slab256Bytes => self.slab_64_bytes.allocate(layout),
            slab::SlabSize::Slab512Bytes => self.slab_64_bytes.allocate(layout),
            slab::SlabSize::Slab1024Bytes => self.slab_64_bytes.allocate(layout),
            slab::SlabSize::Slab2048Bytes => self.slab_64_bytes.allocate(layout),
            slab::SlabSize::Slab4096Bytes => self.slab_64_bytes.allocate(layout),
            _ => unimplemented!(),
        }
    }

    /// Deallocate(free) object.
    pub unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        match Self::get_slab_size(&layout) {
            slab::SlabSize::Slab64Bytes => self.slab_64_bytes.deallocate(ptr),
            slab::SlabSize::Slab128Bytes => self.slab_64_bytes.deallocate(ptr),
            slab::SlabSize::Slab256Bytes => self.slab_64_bytes.deallocate(ptr),
            slab::SlabSize::Slab512Bytes => self.slab_64_bytes.deallocate(ptr),
            slab::SlabSize::Slab1024Bytes => self.slab_64_bytes.deallocate(ptr),
            slab::SlabSize::Slab2048Bytes => self.slab_64_bytes.deallocate(ptr),
            slab::SlabSize::Slab4096Bytes => self.slab_64_bytes.deallocate(ptr),
            _ => unimplemented!(),
        }
    }

    /// Convert `layout.size` to `SlabSize`
    fn get_slab_size(layout: &Layout) -> SlabSize {
        let slab_size = match layout.size() {
            0..=64 => SlabSize::Slab64Bytes,
            65..=128 => SlabSize::Slab128Bytes,
            129..=256 => SlabSize::Slab256Bytes,
            257..=512 => SlabSize::Slab512Bytes,
            513..=1024 => SlabSize::Slab1024Bytes,
            1025..=2048 => SlabSize::Slab2048Bytes,
            2049..=4096 => SlabSize::Slab4096Bytes,
            _ => panic!("unexpected size"),
        };

        if layout.align() <= slab_size as usize {
            slab_size
        } else {
            // unaligned layout
            SlabSize::Slab4096Bytes
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
