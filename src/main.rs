#![no_std]
#![no_main]
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

global_asm!(
    ".att_syntax prefix",
    ".section .text.entry",
    ".global _start",
    "_start:",
    "xorw %ax, %ax",
    "movw %ax, %ds",
    "movw %ax, %es",
    "movw %ax, %ss",
    "movw $0x7C00, %sp",
    "call {main}",
    main = sym hello_world,
);

#[no_mangle]
fn hello_world() -> ! {
    for &byte in b"Hello, World!" {
        bios_print_char(byte);
    }
    panic!()
}

fn bios_print_char(c: u8) {
    unsafe {
        asm!(
            "int $0x10",
            in("ah") 0x0Eu8,
            in("al") c,
            options(nostack, nomem, att_syntax),
        );
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem, att_syntax));
        }
    }
}
