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

Before using this allocator, you need to init it:
```rust
const HEAP_ADDR: usize = 0x8021_0000;
const HEAP_SIZE: usize = 0x8000;

fn init_allocator() {
    // Initialize global allocator
    unsafe {
        ALLOCATOR.init(HEAP_ADDR, HEAP_SIZE);
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
