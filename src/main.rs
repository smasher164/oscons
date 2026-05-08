#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(
    ".section .text.entry",
    ".global _start",
    "_start:",
    "xor ax, ax",
    "mov ds, ax",
    "mov es, ax",
    "mov ss, ax",
    "mov sp, 0x7C00",
    "call {main}",
    "hlt",
    main = sym kernel_main,
);

#[no_mangle]
fn kernel_main() {
    print(b"Hello, World!");
}

fn print(s: &[u8]) {
    for &byte in s {
        unsafe {
            asm!(
                "int 0x10",
                in("ah") 0x0Eu8,
                in("al") byte,
                options(nostack, nomem),
            );
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
