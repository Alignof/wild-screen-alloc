# wild-screen-alloc
[![Rust](https://github.com/Alignof/wild-screen-alloc/actions/workflows/rust.yml/badge.svg)](https://github.com/Alignof/wild-screen-alloc/actions/workflows/rust.yml)  
A hobby slab allocator for bare-metal rust.

## Usage
Create a static allocator in your root module:
```rust
use wild_screen_alloc::WildScreenAlloc;

#[global_allocator]
static mut ALLOCATOR: WildScreenAlloc = WildScreenAlloc::empty();
```

Before using this allocator, you need to initalize it:
```rust
fn init_allocator() {
    // Initialize global allocator
    let heap_addr: usize = /* calculate heap addr */;
    let heap_size: usize = /* calculate heap size */;
    unsafe {
        ALLOCATOR.init(heap_addr, heap_size);
    }
}
```

See `example/` for more details.

## Reference
- [linked-list-allocator](https://github.com/rust-osdev/linked-list-allocator)  
- [slab\_allocator](https://gitlab.redox-os.org/redox-os/slab_allocator)  

## License
This crate is licensed under MIT.   
See `LICENSE` for details.
