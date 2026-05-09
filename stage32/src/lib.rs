#![no_std]
#![no_main]
// global_asm! has no options(att_syntax); the .att_syntax directive is the
// only way to use AT&T syntax, which Rust flags as "bad_asm_style".
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;

// Reload data segment registers with the protected-mode data segment selector
// (0x10) and call stage32_main. CS was set to 0x08 by the far jump from stage16;
// all other segment registers still hold their real-mode values and must be
// updated before any memory access through them.
global_asm!(
    ".att_syntax prefix",
    ".section .text.entry32",
    ".global stage32_entry",
    "stage32_entry:",
    ".code32",
    "movw $0x10, %ax",
    "movw %ax, %ds",
    "movw %ax, %es",
    "movw %ax, %ss",
    "call {main}",
    main = sym stage32_main,
);

const VGA: *mut u16 = 0xB8000 as *mut u16;
const WHITE_ON_BLACK: u8 = 0x0F; // VGA attribute byte: high nibble = background (0=black), low nibble = foreground (F=bright white)

#[no_mangle]
fn stage32_main() -> ! {
    print("Successfully entered protected mode.");
    panic!();
}

fn print(s: &str) {
    for (i, &byte) in s.as_bytes().iter().enumerate() {
        unsafe {
            VGA.add(i).write_volatile(((WHITE_ON_BLACK as u16) << 8) | byte as u16);
        }
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe { asm!("hlt", options(nostack, nomem, att_syntax)); }
    }
}
