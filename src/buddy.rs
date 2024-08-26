//! Implementation of buddy system.
//!
//! ref: [https://github.com/evanw/buddy-malloc](https://github.com/evanw/buddy-malloc)

pub enum BlockSize {
    Byte4K = 4 * 1024, // = PAGE_SIZE
    Byte8K = 8 * 1024,
    Byte16K = 16 * 1024,
    Byte32K = 32 * 1024,
    Byte64K = 64 * 1024,
    Byte128K = 128 * 1024,
    Byte256K = 256 * 1024,
    Byte512K = 512 * 1024,
    Byte1024K = 1024 * 1024,
}

/// Header of memory block
struct Header {
    /// Is memory block used?
    is_used: bool,
    /// Memory block size.
    size: BlockSize,
    /// Next empty node of linked list.
    next: &'static Header,
    /// Buddy address.
    buddy_addr: &'static Header,
}

struct BuddySystem {}