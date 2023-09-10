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
            Some(slab::SlabSize::Slab64Bytes) => self.slab_64_bytes.allocate(),
            Some(slab::SlabSize::Slab128Bytes) => self.slab_128_bytes.allocate(),
            Some(slab::SlabSize::Slab256Bytes) => self.slab_256_bytes.allocate(),
            Some(slab::SlabSize::Slab512Bytes) => self.slab_512_bytes.allocate(),
            Some(slab::SlabSize::Slab1024Bytes) => self.slab_1024_bytes.allocate(),
            Some(slab::SlabSize::Slab2048Bytes) => self.slab_2048_bytes.allocate(),
            Some(slab::SlabSize::Slab4096Bytes) => self.slab_4096_bytes.allocate(),
            None => todo!(),
        }
    }

    /// Deallocate(free) object.
    /// # Safety
    /// Given pointer must be valid.
    pub unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        match Self::get_slab_size(&layout) {
            Some(slab::SlabSize::Slab64Bytes) => self.slab_64_bytes.deallocate(ptr),
            Some(slab::SlabSize::Slab128Bytes) => self.slab_128_bytes.deallocate(ptr),
            Some(slab::SlabSize::Slab256Bytes) => self.slab_256_bytes.deallocate(ptr),
            Some(slab::SlabSize::Slab512Bytes) => self.slab_512_bytes.deallocate(ptr),
            Some(slab::SlabSize::Slab1024Bytes) => self.slab_1024_bytes.deallocate(ptr),
            Some(slab::SlabSize::Slab2048Bytes) => self.slab_2048_bytes.deallocate(ptr),
            Some(slab::SlabSize::Slab4096Bytes) => self.slab_4096_bytes.deallocate(ptr),
            None => todo!(),
        }
    }

    /// Convert `layout.size` to `SlabSize`
    fn get_slab_size(layout: &Layout) -> Option<SlabSize> {
        let slab_size = match layout.size() {
            0..=64 => Some(SlabSize::Slab64Bytes),
            65..=128 => Some(SlabSize::Slab128Bytes),
            129..=256 => Some(SlabSize::Slab256Bytes),
            257..=512 => Some(SlabSize::Slab512Bytes),
            513..=1024 => Some(SlabSize::Slab1024Bytes),
            1025..=2048 => Some(SlabSize::Slab2048Bytes),
            2049..=4096 => Some(SlabSize::Slab4096Bytes),
            _ => None,
        };

        slab_size.map(|size| {
            if layout.align() <= size as usize {
                size
            } else {
                // unaligned layout
                SlabSize::Slab4096Bytes
            }
        })
    }
}

pub struct WildScreenAlloc(Mutex<Option<SlabAllocator>>);

impl WildScreenAlloc {
    /// Return empty `WildScreenAlloc`.
    /// This method exist for to initialize after heap address available.
    /// ```no_run
    /// use wild_screen_alloc::WildScreenAlloc;
    ///
    /// #[global_allocator]
    /// static ALLOCATOR: WildScreenAlloc = WildScreenAlloc::empty();
    ///
    /// pub fn init_heap() { /* initialize ALLOCATOR */ }
    /// ```
    pub const fn empty() -> Self {
        WildScreenAlloc(Mutex::new(None))
    }

    /// Initialize allocator.
    /// ```no_run
    /// use wild_screen_alloc::WildScreenAlloc;
    ///
    /// #[global_allocator]
    /// static mut ALLOCATOR: WildScreenAlloc = WildScreenAlloc::empty();
    ///
    /// pub fn init_heap() {
    ///     let heap_start = 0x8020_0000;
    ///     let heap_size = 0x8000;
    ///     unsafe {
    ///         ALLOCATOR.init(heap_start, heap_size);
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
        WildScreenAlloc(Mutex::new(Some(SlabAllocator::new(start_addr, heap_size))))
    }
}

unsafe impl GlobalAlloc for WildScreenAlloc {
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

    #[test]
    fn alloc_4096_bytes() {
        let dummy_heap = DummyHeap {
            heap_space: [0_u8; HEAP_SIZE],
        };

        unsafe {
            let mut allocator =
                SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
            let size = 4096;
            let layout = Layout::from_size_align(size, size);
            let addr = allocator.allocate(layout.clone().unwrap());
            assert!(!addr.is_null());

            allocator.deallocate(addr, layout.unwrap());
        }
    }

    #[test]
    fn alloc_4097_bytes() {
        let dummy_heap = DummyHeap {
            heap_space: [0_u8; HEAP_SIZE],
        };

        unsafe {
            let mut allocator =
                SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
            let size = 4097;
            let layout = Layout::from_size_align(size, 4096);
            let addr = allocator.allocate(layout.clone().unwrap());
            assert!(addr.is_null());
        }
    }
}
