//! Implementation of slab allocator.
//!
//! It management heap memory over page size.
//!
//! ref: [https://zenn.dev/junjunjunjun/articles/09b8e112c0219c](https://zenn.dev/junjunjunjun/articles/09b8e112c0219c)

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

/// Type of Slab
/// * Full - all objects are allocated.
/// * Partial - some objects are allocated.
/// * Empty - no objects are allocated.
enum SlabKind {
    Full,
    Partial,
    Empty,
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

/// Slab header.
struct SlabHead {
    len: usize,
    _kind: SlabKind,
    head: Option<&'static mut FreeObject>,
    _next: Option<&'static mut Self>,
}

impl SlabHead {
    /// Initialize free objects list and return new `SlabHead`.
    pub unsafe fn new(start_addr: usize, object_size: ObjectSize, num_of_object: usize) -> Self {
        let mut new_list = Self::new_empty(SlabKind::Empty);
        for off in (0..num_of_object).rev() {
            let new_object = (start_addr + off * object_size as usize) as *mut FreeObject;
            new_list.push(&mut *new_object);
        }

        new_list
    }

    /// Return empty head.
    fn new_empty(kind: SlabKind) -> Self {
        SlabHead {
            len: 0,
            _kind: kind,
            head: None,
            _next: None,
        }
    }

    /// Push new free object.
    fn push(&mut self, slab: &'static mut FreeObject) {
        slab.next = self.head.take();
        self.len += 1;
        self.head = Some(slab);
    }

    /// Pop free object.
    fn pop(&mut self) -> Option<&'static mut FreeObject> {
        self.head.take().map(|node| {
            self.head = node.next.take();
            self.len -= 1;
            node
        })
    }
}

/// Linked lists for free slab management.
///
/// It has three lists to match `SlabKind`.  
/// Allocator normally use partial, but it use empty list and move one to partial when partial is empty.
/// Note that only "empty" is used temporarily now. (TODO!)
struct SlabLists {
    _full: SlabHead,
    partial: SlabHead,
    empty: SlabHead,
}

impl SlabLists {
    /// Create new slab lists.
    pub unsafe fn new(start_addr: usize, alloc_size: usize, object_size: ObjectSize) -> Self {
        let num_of_object = alloc_size / object_size as usize;
        assert!(num_of_object > 0);

        SlabLists {
            _full: SlabHead::new_empty(SlabKind::Full),
            partial: SlabHead::new_empty(SlabKind::Partial),
            empty: SlabHead::new(start_addr, object_size, num_of_object),
        }
    }

    /// Get free object from partial
    fn pop_from_partial(&mut self) -> Option<&'static mut FreeObject> {
        self.partial.pop()
    }

    /// Get free object from empty
    fn pop_from_empty(&mut self) -> Option<&'static mut FreeObject> {
        self.empty.pop()
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
    pub unsafe fn new(start_addr: usize, alloc_size: usize, object_size: ObjectSize) -> Self {
        Cache {
            _object_size: object_size,
            slab_lists: SlabLists::new(start_addr, alloc_size, object_size),
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
            self.slab_lists.empty.push(&mut *ptr);
        }
    }
}
