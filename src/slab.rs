//! Implementation of slab allocator.
//!
//! It management heap memory over page size.
//!
//! ref: [https://zenn.dev/junjunjunjun/articles/09b8e112c0219c](https://zenn.dev/junjunjunjun/articles/09b8e112c0219c)

mod list;

use super::constants;
use alloc::alloc::Layout;

/// An enum that indicate size of objects managed by the Slab cache.
#[derive(Copy, Clone)]
pub enum ObjectSize {
    Byte64 = 64,
    Byte128 = 128,
    Byte256 = 256,
    Byte512 = 512,
    Byte1024 = 1024,
    Byte2048 = 2048,
    Byte4096 = 4096, // 4 kB = PAGE_SIZE
}

/// A linked list managing free objects.
/// This struct is placed unused heap space.
struct FreeObject {
    next: Option<&'static mut Self>,
}

impl FreeObject {
    /// Return address itself.
    fn addr(&self) -> usize {
        self as *const _ as usize
    }
}

/// Slab (= 1 PAGE memory block)
///
/// # Memory layout
/// ```ignore
/// ┌──────────────────────────────────────────────────────────────────────────┐
/// │                                                                          │
/// │                           size_of::<Node>()                              │
/// │   ◄──────────────────────────────────────────────────────────────────►   │
/// │                                    size_of::<Slab>()                     │
/// │  0              ◄────────────────────────────────────────────────────►   │
/// │  ┌─────────────┬────────┬────────────┬──────────────┬─────────────────┐  │
/// │  │  Node.next  │  kind  │  obj_size  │  used_bytes  │  free_obj_head ─┼──┘
/// │  ├─────────────┼────────┴────┬───────┴─────┬────────┴───┬─────────────┤   
/// └──┼► free_obj   │  free_obj   │  free_obj   │  free_obj  │  free_obj   │   
///    ├─────────────┼─────────────┼─────────────┼────────────┼─────────────┤   
///    │  free_obj   │  free_obj   │  free_obj   │  free_obj  │  free_obj   │   
///    └─────────────┴─────────────┴─────────────┴────────────┴─────────────┘   
///                                                                       4096
/// ```
#[repr(C)]
struct Slab {
    pub kind: SlabKind,
    obj_size: ObjectSize,
    used_bytes: usize,
    free_obj_head: Option<&'static mut FreeObject>,
}

impl Slab {
    /// Return empty object Slab
    fn new_empty(kind: SlabKind, obj_size: ObjectSize) -> Self {
        Slab {
            kind,
            obj_size,
            used_bytes: 0,
            free_obj_head: None,
        }
    }

    /// Initialize free objects list and return new `SlabHead`.
    pub unsafe fn new(object_size: ObjectSize, free_obj_start_addr: usize) -> Self {
        let num_of_object = (constants::PAGE_SIZE - size_of::<Slab>()) / object_size as usize;
        assert!(num_of_object > 0);

        let mut new_list = Self::new_empty(SlabKind::Empty, object_size);
        for off in (0..num_of_object).rev() {
            let new_object = (free_obj_start_addr + off * object_size as usize) as *mut FreeObject;
            new_list.push(&mut *new_object);
        }

        new_list
    }

    /// Push new free object.
    fn push(&mut self, slab: &'static mut FreeObject) {
        slab.next = self.free_obj_head.take();
        self.used_bytes += self.obj_size as usize;
        self.free_obj_head = Some(slab);
    }

    /// Pop free object.
    fn pop(&mut self) -> Option<&'static mut FreeObject> {
        self.free_obj_head.take().map(|node| {
            self.free_obj_head = node.next.take();
            self.used_bytes -= self.obj_size as usize;
            node
        })
    }
}

/// Type of Slab
enum SlabKind {
    /// All objects are allocated.
    Full,
    /// Some objects are allocated.
    Partial,
    /// No objects are allocated.
    Empty,
}

/// Linked lists for free slab management.
///
/// It has three lists to match `SlabKind`.  
/// Allocator normally use partial, but it use empty list and move one to partial when partial is empty.
/// Note that only "empty" is used temporarily now. (TODO!)
struct SlabLists {
    full: list::List,
    partial: list::List,
    empty: list::List,
}

impl SlabLists {
    /// Create new slab lists.
    pub unsafe fn new(object_size: ObjectSize) -> Self {
        SlabLists {
            full: list::List::new_empty(),
            partial: list::List::new_empty(),
            empty: list::List::new(object_size, constants::DEFAULT_SLAB_NUM),
        }
    }

    /// Get free object from partial
    fn pop_from_partial(&mut self) -> Option<&'static mut FreeObject> {
        // TODO
        self.partial.head.as_mut().unwrap().pop()
    }

    /// Get free object from empty
    fn pop_from_empty(&mut self) -> Option<&'static mut FreeObject> {
        // TODO
        self.empty.head.as_mut().unwrap().pop()
    }
}

/// Cache that contains slab lists.
pub struct Cache {
    /// Size of object. (e.g. 64byte, 128byte)
    _object_size: ObjectSize,
    /// slab's linked list
    slab_lists: SlabLists,
}

impl Cache {
    /// Create new slab cache.
    pub unsafe fn new(object_size: ObjectSize) -> Self {
        Cache {
            _object_size: object_size,
            slab_lists: SlabLists::new(object_size),
        }
    }

    /// Return object address according to `layout.size`.
    pub fn allocate(&mut self) -> *mut u8 {
        match self.slab_lists.pop_from_partial() {
            Some(object) => object.addr() as *mut u8,
            None => match self.slab_lists.pop_from_empty() {
                Some(object) => object.addr() as *mut u8,
                None => core::ptr::null_mut(),
            },
        }
    }

    /// Free object according to `layout.size`.
    pub fn deallocate(&mut self, ptr: *mut u8) {
        let ptr = ptr.cast::<FreeObject>();
        unsafe {
            // TODO
            self.slab_lists.empty.head.as_mut().unwrap().push(&mut *ptr);
        }
    }
}

/// Slab allocator that provide global allocator.
/// If allocate size over 4096 bytes, it delegate to `linked_list_allocator`.
pub struct SlabAllocator {
    slab_64_bytes: Cache,
    slab_128_bytes: Cache,
    slab_256_bytes: Cache,
    slab_512_bytes: Cache,
    slab_1024_bytes: Cache,
    slab_2048_bytes: Cache,
    slab_4096_bytes: Cache,
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
            slab_64_bytes: Cache::new(ObjectSize::Byte64),
            slab_128_bytes: Cache::new(ObjectSize::Byte128),
            slab_256_bytes: Cache::new(ObjectSize::Byte256),
            slab_512_bytes: Cache::new(ObjectSize::Byte512),
            slab_1024_bytes: Cache::new(ObjectSize::Byte1024),
            slab_2048_bytes: Cache::new(ObjectSize::Byte2048),
            slab_4096_bytes: Cache::new(ObjectSize::Byte4096),
        }
    }

    /// Allocates a new object.
    pub fn allocate(&mut self, layout: Layout) -> *mut u8 {
        match Self::get_slab_size(&layout) {
            ObjectSize::Byte64 => self.slab_64_bytes.allocate(),
            ObjectSize::Byte128 => self.slab_128_bytes.allocate(),
            ObjectSize::Byte256 => self.slab_256_bytes.allocate(),
            ObjectSize::Byte512 => self.slab_512_bytes.allocate(),
            ObjectSize::Byte1024 => self.slab_1024_bytes.allocate(),
            ObjectSize::Byte2048 => self.slab_2048_bytes.allocate(),
            ObjectSize::Byte4096 => self.slab_4096_bytes.allocate(),
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
            ObjectSize::Byte64 => self.slab_64_bytes.deallocate(ptr),
            ObjectSize::Byte128 => self.slab_128_bytes.deallocate(ptr),
            ObjectSize::Byte256 => self.slab_256_bytes.deallocate(ptr),
            ObjectSize::Byte512 => self.slab_512_bytes.deallocate(ptr),
            ObjectSize::Byte1024 => self.slab_1024_bytes.deallocate(ptr),
            ObjectSize::Byte2048 => self.slab_2048_bytes.deallocate(ptr),
            ObjectSize::Byte4096 => self.slab_4096_bytes.deallocate(ptr),
        }
    }

    /// Convert `layout.size` to `ObjectSize`
    fn get_slab_size(layout: &Layout) -> ObjectSize {
        assert!(layout.size() < 4096);
        match layout.size() {
            0..=64 => ObjectSize::Byte64,
            65..=128 => ObjectSize::Byte128,
            129..=256 => ObjectSize::Byte256,
            257..=512 => ObjectSize::Byte512,
            513..=1024 => ObjectSize::Byte1024,
            1025..=2048 => ObjectSize::Byte2048,
            2049..4096 => ObjectSize::Byte4096,
            _ => unreachable!(),
        }
    }
}
