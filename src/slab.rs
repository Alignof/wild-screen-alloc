#[derive(Copy, Clone)]
pub enum SlabSize {
    Slab64Bytes,
    Slab128Bytes,
    Slab256Bytes,
    Slab512Bytes,
    Slab1024Bytes,
    Slab2048Bytes,
    Slab4096Bytes,
}

enum SlabKind {
    Full,
    Partial,
    Empty,
}

struct SlabHead {
    kind: SlabKind,
    next: Option<&'static mut Self>,
}

struct SlabList {
    len: usize,
    head: Option<&'static mut SlabHead>,
}

impl SlabList {
    pub unsafe fn new(start_addr: usize, object_size: SlabSize, num_of_object: usize) -> Self {
        let head_slab_addr = ((start_addr + core::mem::size_of::<SlabHead>()) as *const u8)
            .align_offset(object_size as usize) as usize;

        let mut new_list = Self::new_empty();
        for off in (0..num_of_object as usize).rev() {
            let new_object = (head_slab_addr + off * object_size as usize) as *mut SlabHead;
            new_list.push(&mut *new_object);
        }

        new_list
    }

    fn push(&mut self, slab: &'static mut SlabHead) {
        slab.next = self.head.take();
        self.len += 1;
        self.head = Some(slab);
    }

    pub fn new_empty() -> Self {
        SlabList { len: 0, head: None }
    }
}

struct SlabFreeList {
    full: SlabList,
    partial: SlabList,
    empty: SlabList,
}

impl SlabFreeList {
    pub unsafe fn new(start_addr: usize, alloc_size: usize, object_size: SlabSize) -> Self {
        let num_of_object = (alloc_size - core::mem::size_of::<SlabHead>()) / object_size as usize;
        assert!(num_of_object > 0);

        SlabFreeList {
            full: SlabList::new_empty(),
            partial: SlabList::new_empty(),
            empty: SlabList::new(start_addr, object_size, num_of_object),
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
