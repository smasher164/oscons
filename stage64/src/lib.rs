#![no_std]
#![no_main]
// global_asm! has no options(att_syntax); the .att_syntax directive is the
// only way to use AT&T syntax, which Rust flags as "bad_asm_style".
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use oscons::Vga;

// CS was set to 0x18 by the far jump from stage32; clear the data segment
// registers that stage32 set to 0x10 — they are vestigial in 64-bit mode.
global_asm!(
    ".att_syntax prefix",
    ".section .text.entry64",
    ".global stage64_entry",
    "stage64_entry:",
    ".code64",
    "xorw %ax, %ax",
    "movw %ax, %ds",
    "movw %ax, %es",
    "movw %ax, %ss",
    "call {main}",
    main = sym stage64_main,
);

// Defined in stage32/src/lib.rs; row/col reflect where stage32 stopped printing.
extern "C" {
    static mut VGA: Vga;
}

#[no_mangle]
extern "C" fn stage64_main() -> ! {
    let vga = unsafe { &mut *(&raw mut VGA) };
    let _ = write!(vga, "Entered long mode.");
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
