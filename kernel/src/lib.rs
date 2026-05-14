#![no_std]
#![no_main]
#![allow(bad_asm_style)]

use core::arch::asm;
use core::fmt::Write;
use core::panic::PanicInfo;
use oscons::{BootInfo, Vga};

#[no_mangle]
extern "C" fn kernel_entry(_info: *const BootInfo) -> ! {
    let mut vga = Vga { row: 0, col: 0 };
    vga.clear();
    let _ = write!(vga, "Hello from the kernel!");
    panic!()
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem, att_syntax));
        }
    }
}
