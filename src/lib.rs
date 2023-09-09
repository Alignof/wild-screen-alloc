#![no_std]

extern crate alloc;

mod slab;

use alloc::alloc::{GlobalAlloc, Layout};
use slab::{SlabCache, SlabSize};
use spin::Mutex;

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
    /// Return new `SlabAllocator`.
    /// # Safety
    /// `start_addr` must be aligned 4096.
    ///
    /// # Panics
    /// If `start_addr` isn't aligned 4096, this function will panic.
    #[must_use]
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
            slab::SlabSize::Slab64Bytes => self.slab_64_bytes.allocate(),
            slab::SlabSize::Slab128Bytes => self.slab_128_bytes.allocate(),
            slab::SlabSize::Slab256Bytes => self.slab_256_bytes.allocate(),
            slab::SlabSize::Slab512Bytes => self.slab_512_bytes.allocate(),
            slab::SlabSize::Slab1024Bytes => self.slab_1024_bytes.allocate(),
            slab::SlabSize::Slab2048Bytes => self.slab_2048_bytes.allocate(),
            slab::SlabSize::Slab4096Bytes => self.slab_4096_bytes.allocate(),
        }
    }

    /// Deallocate(free) object.
    /// # Safety
    /// Given pointer must be valid.
    pub unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        match Self::get_slab_size(&layout) {
            slab::SlabSize::Slab64Bytes => self.slab_64_bytes.deallocate(ptr),
            slab::SlabSize::Slab128Bytes => self.slab_128_bytes.deallocate(ptr),
            slab::SlabSize::Slab256Bytes => self.slab_256_bytes.deallocate(ptr),
            slab::SlabSize::Slab512Bytes => self.slab_512_bytes.deallocate(ptr),
            slab::SlabSize::Slab1024Bytes => self.slab_1024_bytes.deallocate(ptr),
            slab::SlabSize::Slab2048Bytes => self.slab_2048_bytes.deallocate(ptr),
            slab::SlabSize::Slab4096Bytes => self.slab_4096_bytes.deallocate(ptr),
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

pub struct LockedAllocator(Mutex<Option<SlabAllocator>>);

impl LockedAllocator {
    /// Return empty `LockedAllocator`.
    /// This method exist for to initialize after heap address available.
    /// ```
    /// use wild_scree_alloc::LockedAllocator;
    ///
    /// #[global_allocator]
    /// static ALLOCATOR: LockedAllocator = LockedAllocator::empty();
    ///
    /// pub fn init_heap() { /* initialize ALLOCATOR */ }
    /// ```
    pub const fn empty() -> Self {
        LockedAllocator(Mutex::new(None))
    }

    /// Initialize allocator.
    /// ```
    /// pub fn init_heap() {
    ///     let heap_start = ...;
    ///     let heap_end = ...;
    ///     let heap_size = heap_end - heap_start;
    ///     unsafe {
    ///         ALLOCATOR.lock().init(heap_start, heap_size);
    ///     }
    /// }
    /// ```
    ///
    /// # Safety
    /// `start_addr` must be aligned 4096.
    pub unsafe fn init(&mut self, start_addr: usize, heap_size: usize) {
        *self.0.lock() = Some(SlabAllocator::new(start_addr, heap_size))
    }

    /// Create new allocator locked by mutex.
    /// # Safety
    /// `start_addr` must be aligned 4096.
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        LockedAllocator(Mutex::new(Some(SlabAllocator::new(start_addr, heap_size))))
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    /// Just call `SlabAllocator::allocte`.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match *self.0.lock() {
            Some(ref mut allocator) => allocator.allocate(layout),
            None => panic!("The allocator is not initialized"),
        }
    }

    /// Just call `SlabAllocator::deallocate`.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match *self.0.lock() {
            Some(ref mut allocator) => allocator.deallocate(ptr, layout),
            None => panic!("The allocator is not initialized"),
        }
    }
}

#[cfg(test)]
mod alloc_tests {
    use crate::{constants, SlabAllocator};
    use alloc::alloc::Layout;
    use core::mem::{align_of, size_of};

    const HEAP_SIZE: usize = 8 * constants::PAGE_SIZE;
    #[repr(align(4096))]
    struct DummyHeap {
        heap_space: [u8; HEAP_SIZE],
    }

    #[test]
    fn create_allocator() {
        let dummy_heap = DummyHeap {
            heap_space: [0_u8; HEAP_SIZE],
        };

        unsafe {
            let _ = SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
        }
    }

    #[test]
    fn alloc_and_free_test() {
        let dummy_heap = DummyHeap {
            heap_space: [0_u8; HEAP_SIZE],
        };

        unsafe {
            let mut allocator =
                SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
            let size = size_of::<usize>() * 2;
            let layout = Layout::from_size_align(size, align_of::<usize>());
            let addr = allocator.allocate(layout.clone().unwrap());
            assert!(!addr.is_null());

            allocator.deallocate(addr, layout.unwrap());
        }
    }
}
