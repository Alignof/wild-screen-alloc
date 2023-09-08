#[derive(Copy, Clone)]
pub enum SlabSize {
    Slab64Bytes = 64,
    Slab128Bytes = 128,
    Slab256Bytes = 256,
    Slab512Bytes = 512,
    Slab1024Bytes = 1024,
    Slab2048Bytes = 2048,
    Slab4096Bytes = 4096,
}

enum SlabKind {
    Full,
    Partial,
    Empty,
}

struct FreeObject {
    next: Option<&'static mut Self>,
}

struct SlabHead {
    len: usize,
    kind: SlabKind,
    head: Option<&'static mut FreeObject>,
    next: Option<&'static mut Self>,
}

impl SlabHead {
    pub unsafe fn new(start_addr: usize, object_size: SlabSize, num_of_object: usize) -> Self {
        let mut new_list = Self::new_empty(SlabKind::Empty);
        for off in (0..num_of_object as usize).rev() {
            let new_object = (start_addr + off * object_size as usize) as *mut FreeObject;
            new_list.push(&mut *new_object);
        }

        new_list
    }

    fn push(&mut self, slab: &'static mut FreeObject) {
        slab.next = self.head.take();
        self.len += 1;
        self.head = Some(slab);
    }

    pub fn new_empty(kind: SlabKind) -> Self {
        SlabHead {
            len: 0,
            kind,
            head: None,
            next: None,
        }
    }
}

struct SlabFreeList {
    full: SlabHead,
    partial: SlabHead,
    empty: SlabHead,
}

impl SlabFreeList {
    pub unsafe fn new(start_addr: usize, alloc_size: usize, object_size: SlabSize) -> Self {
        let num_of_object = alloc_size / object_size as usize;
        assert!(num_of_object > 0);

        SlabFreeList {
            full: SlabHead::new_empty(SlabKind::Full),
            partial: SlabHead::new_empty(SlabKind::Partial),
            empty: SlabHead::new(start_addr, object_size, num_of_object),
        }
    }
}

pub struct SlabCache {
    object_size: SlabSize,
    slab_free_list: SlabFreeList,
}

impl SlabCache {
    pub unsafe fn new(start_addr: usize, alloc_size: usize, object_size: SlabSize) -> Self {
        SlabCache {
            object_size,
            slab_free_list: SlabFreeList::new(start_addr, alloc_size, object_size),
        }
    }
}
