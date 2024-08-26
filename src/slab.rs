//! Implementation of slab allocator.
//!
//! It management heap memory over page size.
//!
//! ref: [https://zenn.dev/junjunjunjun/articles/09b8e112c0219c](https://zenn.dev/junjunjunjun/articles/09b8e112c0219c)

mod list;

use super::constants;

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
/// └───► free_obj   │  free_obj   │  free_obj   │  free_obj  │  free_obj   │   
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
