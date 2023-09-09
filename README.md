# wild-screen-alloc
A hobby slab allocator for bare-metal rust.

## Example
何なのよ，𝒘𝒊(𝒍)𝒅-𝒔𝒄𝒓𝒆𝒆𝒏 𝗮𝗹𝗹𝗼𝗰って．
```rust
#![no_std]
#![no_main]

extern crate alloc;
extern crate panic_halt;
extern crate wild_screen_alloc;

use alloc::format;
use alloc::vec::Vec;
use riscv_rt::entry;
use wild_screen_alloc::WildScreenAlloc;

const UART_ADDR: *mut u32 = 0x1001_0000 as *mut u32;
const HEAP_ADDR: usize = 0x8021_0000;
const HEAP_SIZE: usize = 8 * 4096;

#[global_allocator]
static mut ALLOCATOR: WildScreenAlloc = WildScreenAlloc::empty();

unsafe fn uart_print(format: &str) {
    for c in format.chars() {
        while (UART_ADDR.read_volatile() as i32) < 0 {}
        UART_ADDR.write_volatile(c as u32);
    }
}

#[entry]
#[allow(clippy::empty_loop)]
fn main() -> ! {
    let mut vector: Vec<u8> = Vec::new();

    // Initialize global allocator
    unsafe {
        ALLOCATOR.init(HEAP_ADDR, HEAP_SIZE);
    }

    for i in 0..10 {
        vector.push(i);
    }

    for num in &vector {
        unsafe {
            uart_print(&format!("{num}\n"));
        }
    }

    let sum = vector.iter().sum::<u8>();
    unsafe {
        uart_print(&format!("----------\n"));
        uart_print(&format!("sum: {sum}\n"));
    }

    loop {}
}

/* $ cargo r
 * 0
 * 1
 * 2
 * 3
 * 4
 * 5
 * 6
 * 7
 * 8
 * 9
 * ----------
 * sum: 45
*/
```

## Reference
- [linked-list-allocator](https://github.com/rust-osdev/linked-list-allocator)  
- [slab\_allocator](https://gitlab.redox-os.org/redox-os/slab_allocator)  

## License
This crate is licensed under MIT.   
See `LICENSE` for details.
