#![no_std]
#![no_main]
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::fmt::Write;
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

// Represents the BIOS TTY output, available in real mode only. Each character
// is written via interrupt 0x10 (video services).
struct Tty;

impl Write for Tty {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            bios_print_char(byte);
        }
        Ok(())
    }
}

#[no_mangle]
extern "C" fn hello_world() -> ! {
    let _ = Tty.write_str("Hello, World!");
    panic!()
}

fn bios_print_char(c: u8) {
    unsafe {
        asm!(
            "int $0x10",
            in("ah") 0x0Eu8, // AH=0x0E: TTY output function
            in("al") c,      // AL: character to print
            options(nostack, att_syntax),
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
