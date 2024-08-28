//! Implementation of slab allocator.
//!
//! It management heap memory over page size.
//!
//! ref: [https://zenn.dev/junjunjunjun/articles/09b8e112c0219c](https://zenn.dev/junjunjunjun/articles/09b8e112c0219c)

mod list;

use super::constants;
use crate::buddy;

use alloc::alloc::Layout;
use alloc::sync::Arc;
use core::cell::OnceCell;
use spin::Mutex;

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
/// Node of `list::List`
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
/// │  │    next     │  kind  │  obj_size  │  used_bytes  │  free_obj_head ─┼──┘
/// │  ├─────────────┼────────┴────┬───────┴─────┬────────┴───┬─────────────┤   
/// └──┼► free_obj   │  free_obj   │  free_obj   │  free_obj  │  free_obj   │   
///    ├─────────────┼─────────────┼─────────────┼────────────┼─────────────┤   
///    │  free_obj   │  free_obj   │  free_obj   │  free_obj  │  free_obj   │   
///    └─────────────┴─────────────┴─────────────┴────────────┴─────────────┘   
///                                                                       4096
/// ```
#[repr(C)]
struct Slab {
    /// Slab kind.
    pub kind: SlabKind,
    /// Managing object size.
    obj_size: ObjectSize,
    /// Used size (unit: byte).
    used_bytes: usize,
    /// Next node pointer
    next: Option<&'static mut Self>,
    /// Head pointer of linked free object list.
    free_obj_head: Option<&'static mut FreeObject>,
}

impl Slab {
    /// Return empty object Slab
    fn new_empty(kind: SlabKind, obj_size: ObjectSize) -> Self {
        Slab {
            kind,
            obj_size,
            used_bytes: 0,
            next: None,
            free_obj_head: None,
        }
    }

    /// Initialize free objects list and return new `SlabHead`.
    pub unsafe fn new(object_size: ObjectSize, allocated_page_ptr: *mut Self) -> &'static mut Self {
        let free_obj_start_addr =
            unsafe { allocated_page_ptr.byte_add(size_of::<Self>()) as usize };
        let num_of_object = (constants::PAGE_SIZE - size_of::<Slab>()) / object_size as usize;
        assert!(num_of_object > 0);

        let new_slab = unsafe {
            *allocated_page_ptr = Self::new_empty(SlabKind::Empty, object_size);
            allocated_page_ptr
        };

        for off in (0..num_of_object).rev() {
            let new_object = (free_obj_start_addr + off * object_size as usize) as *mut FreeObject;
            (*new_slab).push(&mut *new_object);
        }

        unsafe { &mut *new_slab }
    }

    /// Push new free object.
    fn push(&mut self, obj: &'static mut FreeObject) {
        obj.next = self.free_obj_head.take();
        self.used_bytes += self.obj_size as usize;
        self.free_obj_head = Some(obj);
    }

    /// Pop free object.
    fn pop(&mut self) -> Option<&'static mut FreeObject> {
        self.free_obj_head.take().map(|node| {
            self.free_obj_head = node.next.take();
            self.used_bytes -= self.obj_size as usize;
            node
        })
    }

    fn is_contain(&self, obj_ptr: *const FreeObject) -> bool {
        let slab_start = self as *const Self as usize;
        let slab_end = unsafe { (self as *const Self).byte_add(constants::PAGE_SIZE) as usize };

        (slab_start..slab_end).contains(&(obj_ptr as usize))
    }
}

/// Type of Slab
#[derive(Copy, Clone)]
enum SlabKind {
    /// All objects are allocated.
    Full,
    /// Some objects are allocated.
    Partial,
    /// No objects are allocated.
    Empty,
}

/// Cache that contains slab lists.
///
/// It has three lists to match `SlabKind`.  
/// Allocator normally use partial, but it use empty list and move one to partial when partial is empty.
/// Note that only "empty" is used temporarily now. (TODO!)
pub struct Cache {
    /// Size of object. (e.g. 64byte, 128byte)
    object_size: ObjectSize,
    /// Page allocator for create new `Empty` node.
    page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    /// All objects are allocated.
    full: list::FullList,
    /// Some objects are allocated.
    partial: list::PartialList,
    /// None of objects are allocated.
    empty: list::EmptyList,
}

impl Cache {
    /// Create new slab cache.
    pub unsafe fn new(
        object_size: ObjectSize,
        page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        let empty = list::EmptyList::new(
            object_size,
            constants::DEFAULT_SLAB_NUM,
            page_allocator.clone(),
        );

        Cache {
            object_size,
            page_allocator,
            full: list::FullList::new_empty(),
            partial: list::PartialList::new_empty(),
            empty,
        }
    }

    /// Move `Slab` to corresponding list.
    fn slab_migrate(&mut self, slab_ref: &'static mut Slab, dst_kind: SlabKind) {
        // change slab kind
        slab_ref.kind = dst_kind;

        // append slab
        match dst_kind {
            SlabKind::Full => self.full.push_slab(slab_ref),
            SlabKind::Partial => self.partial.push_slab(slab_ref),
            SlabKind::Empty => self.empty.push_slab(slab_ref),
        }
    }

    /// Return object address according to `layout.size`.
    pub fn allocate(&mut self) -> *mut u8 {
        match self.partial.peek() {
            Some(partial_slab_ptr) => unsafe {
                match (*partial_slab_ptr).pop() {
                    Some(obj) => obj as *mut FreeObject as *mut u8,
                    None => {
                        // partial -> full
                        let full_slab = self.partial.pop_slab().unwrap();
                        self.slab_migrate(full_slab, SlabKind::Full);

                        self.allocate() // retry
                    }
                }
            },
            None => {
                // empty -> partial
                let empty_slab = self
                    .empty
                    .pop_slab(self.object_size, self.page_allocator.clone());
                self.slab_migrate(empty_slab, SlabKind::Full);
                self.allocate() // retry
            }
        }
    }

    /// Free object according to `layout.size`.
    pub fn deallocate(&mut self, ptr: *mut u8) {
        let obj_ptr = ptr.cast::<FreeObject>();

        match self.partial.pop_corresponding_slab(obj_ptr) {
            Some(partial_slab) => unsafe {
                partial_slab.push(&mut *obj_ptr);

                if partial_slab.used_bytes == 0 {
                    // partial -> empty
                    self.slab_migrate(partial_slab, SlabKind::Empty);
                } else {
                    // push back poped slab.
                    self.partial.push_slab(partial_slab);
                }
            },
            None => match self.full.pop_corresponding_slab(obj_ptr) {
                Some(full_slab) => unsafe {
                    full_slab.push(&mut *obj_ptr);

                    // full -> partial
                    self.slab_migrate(full_slab, SlabKind::Partial);
                },
                None => panic!("corresponding slab is not found"),
            },
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
    pub unsafe fn new(
        _start_addr: usize,
        _heap_size: usize,
        page_allocator: Arc<Mutex<OnceCell<buddy::BuddySystem>>>,
    ) -> Self {
        SlabAllocator {
            slab_64_bytes: Cache::new(ObjectSize::Byte64, page_allocator.clone()),
            slab_128_bytes: Cache::new(ObjectSize::Byte128, page_allocator.clone()),
            slab_256_bytes: Cache::new(ObjectSize::Byte256, page_allocator.clone()),
            slab_512_bytes: Cache::new(ObjectSize::Byte512, page_allocator.clone()),
            slab_1024_bytes: Cache::new(ObjectSize::Byte1024, page_allocator.clone()),
            slab_2048_bytes: Cache::new(ObjectSize::Byte2048, page_allocator.clone()),
            slab_4096_bytes: Cache::new(ObjectSize::Byte4096, page_allocator.clone()),
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
