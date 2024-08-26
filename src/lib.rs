#![no_std]

extern crate alloc;
extern crate linked_list_allocator;

mod slab;

use alloc::alloc::{GlobalAlloc, Layout};
use core::cell::OnceCell;
use spin::Mutex;

/// Constants.
mod constants {
    /// Number of slab.
    pub const DEFAULT_SLAB_NUM: usize = 8;
    /// Page size.
    pub const PAGE_SIZE: usize = 4096;
}

/// Slab allocator that provide global allocator.
/// If allocate size over 4096 bytes, it delegate to `linked_list_allocator`.
pub struct SlabAllocator {
    slab_64_bytes: slab::Cache,
    slab_128_bytes: slab::Cache,
    slab_256_bytes: slab::Cache,
    slab_512_bytes: slab::Cache,
    slab_1024_bytes: slab::Cache,
    slab_2048_bytes: slab::Cache,
    slab_4096_bytes: slab::Cache,
}

impl SlabAllocator {
    /// Return new `SlabAllocator`.
    /// # Safety
    /// `start_addr` must be aligned 4096.
    ///
    /// # Panics
    /// If `start_addr` isn't aligned 4096, this function will panic.
    #[must_use]
    pub unsafe fn new(_start_addr: usize, _heap_size: usize) -> Self {
        SlabAllocator {
            slab_64_bytes: slab::Cache::new(slab::ObjectSize::Byte64),
            slab_128_bytes: slab::Cache::new(slab::ObjectSize::Byte128),
            slab_256_bytes: slab::Cache::new(slab::ObjectSize::Byte256),
            slab_512_bytes: slab::Cache::new(slab::ObjectSize::Byte512),
            slab_1024_bytes: slab::Cache::new(slab::ObjectSize::Byte1024),
            slab_2048_bytes: slab::Cache::new(slab::ObjectSize::Byte2048),
            slab_4096_bytes: slab::Cache::new(slab::ObjectSize::Byte4096),
        }
    }

    /// Allocates a new object.
    pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
        match Self::get_slab_size(&layout) {
            slab::ObjectSize::Byte64 => self.slab_64_bytes.allocate(),
            slab::ObjectSize::Byte128 => self.slab_128_bytes.allocate(),
            slab::ObjectSize::Byte256 => self.slab_256_bytes.allocate(),
            slab::ObjectSize::Byte512 => self.slab_512_bytes.allocate(),
            slab::ObjectSize::Byte1024 => self.slab_1024_bytes.allocate(),
            slab::ObjectSize::Byte2048 => self.slab_2048_bytes.allocate(),
            slab::ObjectSize::Byte4096 => self.slab_4096_bytes.allocate(),
        }
    }

    /// Deallocate(free) object.
    /// # Safety
    /// Given pointer must be valid.
    ///
    /// # Panics
    /// If given ptr is null, it will panic.
    pub unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        match Self::get_slab_size(&layout) {
            slab::ObjectSize::Byte64 => self.slab_64_bytes.deallocate(ptr),
            slab::ObjectSize::Byte128 => self.slab_128_bytes.deallocate(ptr),
            slab::ObjectSize::Byte256 => self.slab_256_bytes.deallocate(ptr),
            slab::ObjectSize::Byte512 => self.slab_512_bytes.deallocate(ptr),
            slab::ObjectSize::Byte1024 => self.slab_1024_bytes.deallocate(ptr),
            slab::ObjectSize::Byte2048 => self.slab_2048_bytes.deallocate(ptr),
            slab::ObjectSize::Byte4096 => self.slab_4096_bytes.deallocate(ptr),
        }
    }

    /// Convert `layout.size` to `slab::ObjectSize`
    fn get_slab_size(layout: &Layout) -> slab::ObjectSize {
        assert!(layout.size() < 4096);
        match layout.size() {
            0..=64 => slab::ObjectSize::Byte64,
            65..=128 => slab::ObjectSize::Byte128,
            129..=256 => slab::ObjectSize::Byte256,
            257..=512 => slab::ObjectSize::Byte512,
            513..=1024 => slab::ObjectSize::Byte1024,
            1025..=2048 => slab::ObjectSize::Byte2048,
            2049..4096 => slab::ObjectSize::Byte4096,
            _ => unreachable!(),
        }
    }
}

pub struct WildScreenAlloc {
    slab: Mutex<OnceCell<SlabAllocator>>,
}

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
        WildScreenAlloc {
            slab: Mutex::new(OnceCell::new()),
        }
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
        self.slab
            .lock()
            .get_or_init(|| SlabAllocator::new(start_addr, heap_size));
    }

    /// Create new allocator locked by mutex.
    /// # Safety
    /// `start_addr` must be aligned 4096.
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        let new_slab = OnceCell::new();
        new_slab
            .set(SlabAllocator::new(start_addr, heap_size))
            .unwrap_or_else(|_| panic!("SlabAllocator initialization failed"));

        WildScreenAlloc {
            slab: Mutex::new(new_slab),
        }
    }
}

unsafe impl GlobalAlloc for WildScreenAlloc {
    /// Just call `SlabAllocator::allocte`.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.slab.lock().get_mut() {
            Some(ref mut allocator) => allocator.allocate(layout),
            None => panic!("The allocator is not initialized"),
        }
    }

    /// Just call `SlabAllocator::deallocate`.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        match self.slab.lock().get_mut() {
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

    const HEAP_SIZE: usize = 16 * constants::PAGE_SIZE;
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
        let size = size_of::<usize>() * 2;
        let layout = Layout::from_size_align(size, align_of::<usize>());

        unsafe {
            let mut allocator =
                SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
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
        let size = 4096;
        let layout = Layout::from_size_align(size, align_of::<usize>());

        unsafe {
            let mut allocator =
                SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
            let addr = allocator.allocate(layout.clone().unwrap());
            assert!(!addr.is_null());

            allocator.deallocate(addr, layout.unwrap());
        }
    }

    #[test]
    fn alloc_4104_bytes() {
        let dummy_heap = DummyHeap {
            heap_space: [0_u8; HEAP_SIZE],
        };
        let size = 4104;
        let layout = Layout::from_size_align(size, align_of::<usize>());

        unsafe {
            let mut allocator =
                SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
            let addr = allocator.allocate(layout.clone().unwrap());
            assert!(!addr.is_null());

            allocator.deallocate(addr, layout.unwrap());
        }
    }

    #[test]
    fn alloc_8096_bytes() {
        let dummy_heap = DummyHeap {
            heap_space: [0_u8; HEAP_SIZE],
        };
        let size = 8096;
        let layout = Layout::from_size_align(size, align_of::<usize>());

        unsafe {
            let mut allocator =
                SlabAllocator::new(&dummy_heap.heap_space as *const u8 as usize, HEAP_SIZE);
            let addr = allocator.allocate(layout.clone().unwrap());
            assert!(!addr.is_null());

            allocator.deallocate(addr, layout.unwrap());
        }
    }
}
