extern crate alloc;

mod buddy;
mod slab;

use alloc::alloc::{GlobalAlloc, Layout};
use alloc::rc::Rc;
use core::cell::OnceCell;
use spin::Mutex;

/// Constants.
mod constants {
    /// Default number of slab.
    pub const DEFAULT_SLAB_NUM: usize = 8;
    /// Number of buddy size.
    pub const NUM_OF_BUDDY_SIZE: usize = 9;
    /// Page size.
    pub const PAGE_SIZE: usize = 4096;
}

pub struct WildScreenAlloc {
    slab: Mutex<OnceCell<slab::SlabAllocator>>,
    buddy: Rc<Mutex<OnceCell<buddy::BuddySystem>>>,
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
    pub fn empty() -> Self {
        WildScreenAlloc {
            slab: Mutex::new(OnceCell::new()),
            buddy: Rc::new(Mutex::new(OnceCell::new())),
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
        self.buddy
            .lock()
            .get_or_init(|| buddy::BuddySystem::new(start_addr, heap_size));
        self.slab
            .lock()
            .get_or_init(|| slab::SlabAllocator::new(start_addr, heap_size, self.buddy.clone()));
    }

    /// Create new allocator locked by mutex.
    /// # Safety
    /// `start_addr` must be aligned 4096.
    pub unsafe fn new(start_addr: usize, heap_size: usize) -> Self {
        let new_buddy = OnceCell::new();
        new_buddy
            .set(buddy::BuddySystem::new(start_addr, heap_size))
            .unwrap_or_else(|_| panic!("BuddySystem initialization failed"));
        let buddy = Rc::new(Mutex::new(new_buddy));

        let new_slab = OnceCell::new();
        new_slab
            .set(slab::SlabAllocator::new(
                start_addr,
                heap_size,
                buddy.clone(),
            ))
            .unwrap_or_else(|_| panic!("SlabAllocator initialization failed"));

        WildScreenAlloc {
            slab: Mutex::new(new_slab),
            buddy,
        }
    }
}

unsafe impl GlobalAlloc for WildScreenAlloc {
    /// Just call `SlabAllocator::allocte`.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() < 4096 {
            self.slab
                .lock()
                .get_mut()
                .expect("Slab allocator is not initialized")
                .allocate(layout)
        } else {
            self.buddy
                .lock()
                .get_mut()
                .expect("Buddy system is not initialized")
                .allocate(layout)
        }
    }

    /// Just call `SlabAllocator::deallocate`.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() < 4096 {
            self.slab
                .lock()
                .get_mut()
                .expect("Slab allocator is not initialized")
                .deallocate(ptr, layout)
        } else {
            self.buddy
                .lock()
                .get_mut()
                .expect("Buddy system is not initialized")
                .deallocate(ptr, layout)
        }
    }
}

#[cfg(test)]
mod alloc_tests {
    use crate::{buddy::BuddySystem, constants, slab::SlabAllocator};
    use alloc::alloc::Layout;
    use alloc::rc::Rc;
    use core::cell::OnceCell;
    use core::mem::{align_of, size_of};
    use spin::Mutex;

    const HEAP_SIZE: usize =
        2 * constants::NUM_OF_BUDDY_SIZE * constants::DEFAULT_SLAB_NUM * constants::PAGE_SIZE;
    #[repr(C, align(0x20000))]
    struct PageMemoryBlock([u8; HEAP_SIZE]);
    impl Default for PageMemoryBlock {
        fn default() -> Self {
            PageMemoryBlock([0u8; HEAP_SIZE])
        }
    }

    /// ref: [https://qiita.com/blackenedgold/items/823ab427477e37995ee6](https://qiita.com/blackenedgold/items/823ab427477e37995ee6)
    fn alloc_heap() -> Box<[u8]> {
        let vec_size = HEAP_SIZE / constants::PAGE_SIZE;
        let mut vec = Vec::<PageMemoryBlock>::with_capacity(vec_size);
        vec.resize_with(vec_size, Default::default);
        unsafe {
            let mut data = core::mem::transmute::<_, Vec<u8>>(vec);
            data.set_len(HEAP_SIZE);
            data.into_boxed_slice()
        }
    }

    fn create_allocator() -> (Rc<Mutex<OnceCell<BuddySystem>>>, SlabAllocator) {
        let dummy_heap = alloc_heap();
        let buddy_cell = OnceCell::new();
        buddy_cell.get_or_init(|| unsafe {
            BuddySystem::new(
                dummy_heap.as_ref() as *const [u8] as *const u8 as usize,
                HEAP_SIZE,
            )
        });

        let buddy_system = Rc::new(Mutex::new(buddy_cell));
        let slab_allocator = unsafe {
            SlabAllocator::new(
                dummy_heap.as_ref() as *const [u8] as *const u8 as usize,
                HEAP_SIZE,
                buddy_system.clone(),
            )
        };

        (buddy_system, slab_allocator)
    }

    #[test]
    fn create_allocator_test() {
        dbg!("start");
        let dummy_heap = alloc_heap();
        unsafe {
            let buddy_cell = OnceCell::new();
            buddy_cell.get_or_init(|| {
                BuddySystem::new(
                    dummy_heap.as_ref() as *const [u8] as *const u8 as usize,
                    HEAP_SIZE,
                )
            });

            let buddy_system = Rc::new(Mutex::new(buddy_cell));
            let _ = SlabAllocator::new(
                dummy_heap.as_ref() as *const [u8] as *const u8 as usize,
                HEAP_SIZE,
                buddy_system.clone(),
            );
        }
    }

    #[test]
    fn alloc_and_free_test() {
        let size = size_of::<usize>() * 2;
        let layout = Layout::from_size_align(size, align_of::<usize>());
        let (_buddy_system, mut slab_allocator) = create_allocator();

        let addr = slab_allocator.allocate(layout.clone().unwrap());
        assert!(!addr.is_null());

        unsafe {
            slab_allocator.deallocate(addr, layout.unwrap());
        }
    }

    #[test]
    fn alloc_4096_bytes() {
        let size = 4096;
        let layout = Layout::from_size_align(size, align_of::<usize>());
        let (buddy_system, _slab_allocator) = create_allocator();

        let addr = buddy_system
            .lock()
            .get_mut()
            .unwrap()
            .allocate(layout.clone().unwrap());
        assert!(!addr.is_null());

        unsafe {
            buddy_system
                .lock()
                .get_mut()
                .unwrap()
                .deallocate(addr, layout.unwrap());
        }
    }

    #[test]
    fn alloc_4104_bytes() {
        let size = 4104;
        let layout = Layout::from_size_align(size, align_of::<usize>());
        let (buddy_system, _slab_allocator) = create_allocator();

        let addr = buddy_system
            .lock()
            .get_mut()
            .unwrap()
            .allocate(layout.clone().unwrap());
        assert!(!addr.is_null());

        unsafe {
            buddy_system
                .lock()
                .get_mut()
                .unwrap()
                .deallocate(addr, layout.unwrap());
        }
    }

    #[test]
    fn alloc_8096_bytes() {
        let size = 8096;
        let layout = Layout::from_size_align(size, align_of::<usize>());
        let (buddy_system, _slab_allocator) = create_allocator();

        let addr = buddy_system
            .lock()
            .get_mut()
            .unwrap()
            .allocate(layout.clone().unwrap());
        assert!(!addr.is_null());

        unsafe {
            buddy_system
                .lock()
                .get_mut()
                .unwrap()
                .deallocate(addr, layout.unwrap());
        }
    }
}
