#![no_std]
#![no_main]
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;

// Save drive number before zeroing registers, set up segments and stack, then
// call into Rust.
global_asm!(
    ".att_syntax prefix",
    ".section .text.entry",
    ".global _start",
    "_start:",
    "movb %dl, {drive}",
    "xorw %ax, %ax",
    "movw %ax, %ds",
    "movw %ax, %es",
    "movw %ax, %ss",
    "movw $0x7C00, %sp",
    "call {stage1_main}",
    stage1_main = sym stage1_main,
    drive = sym DRIVE,
);

// Reset segments and stack, then call into Rust for stage 2.
global_asm!(
    ".att_syntax prefix",
    ".section .text.entry2",
    ".global stage2_entry",
    "stage2_entry:",
    "xorw %ax, %ax",
    "movw %ax, %ds",
    "movw %ax, %es",
    "movw %ax, %ss",
    "movw $0x7C00, %sp",
    "call {stage2_main}",
    stage2_main = sym stage2_main,
);

// Label defined in global_asm! above; used to jump to stage 2 by symbol rather
// than hardcoding 0x7E00.
extern "C" {
    fn stage2_entry() -> !;
}

// Represents the BIOS TTY output, available in real mode only. Each character
// is written via interrupt 0x10 (video services).
struct Tty;

impl Write for Tty {
    // In stage 1 section so it is reachable if called before stage 2 is loaded.
    #[link_section = ".text.stage1"]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            bios_print_char(byte);
        }
        Ok(())
    }
}

// BIOS sets DL to the drive number before jumping to 0x7C00. Saved here before
// any function call can clobber it.
#[link_section = ".data.stage1"]
static mut DRIVE: u8 = 0;

#[link_section = ".rodata.stage1"]
static DISK_ERROR: &str = "Error reading from disk.";

#[no_mangle]
#[link_section = ".text.stage1"]
fn stage1_main() -> ! {
    let drive = unsafe { DRIVE };
    for _ in 0..3 {
        if try_disk_read(drive) {
            unsafe { asm!("ljmpw $0, ${0}", sym stage2_entry, options(noreturn, att_syntax)) }
        }
    }
    let _ = Tty.write_str(DISK_ERROR);
    panic!()
}

#[link_section = ".text.stage1"]
fn try_disk_read(drive: u8) -> bool {
    let no_error: u8;
    let sectors_read: u8;
    unsafe {
        asm!(
            "int $0x13",
            "setnc {ok}",
            ok = lateout(reg_byte) no_error,
            in("ah") 2u8,                        // read sectors
            inlateout("al") 1u8 => sectors_read, // 1 sector; al returns count read
            in("ch") 0u8,                        // cylinder 0
            in("cl") 2u8,                        // sector 2
            in("dh") 0u8,                        // head 0
            in("dl") drive,                      // dl already has drive number
            in("bx") 0x7E00u16,                  // load into memory at 0x7e00
            options(att_syntax),
        );
    }
    no_error != 0 && sectors_read == 1
}

#[no_mangle]
fn stage2_main() -> ! {
    let _ = Tty.write_str("Hello from stage 2!");
    panic!()
}

// Lives in stage 1 memory; callable from stage 2 since stage 1 stays in RAM
// after the disk load.
#[link_section = ".text.stage1"]
fn bios_print_char(c: u8) {
    unsafe {
        asm!(
            "int $0x10",
            in("ah") 0x0Eu8, // AH=0x0E: TTY output function
            in("al") c,      // AL: character to print
            options(nostack, nomem, att_syntax),
        );
    }
}

// In stage 1 section so it is reachable if panic fires before stage 2 is loaded.
#[panic_handler]
#[link_section = ".text.stage1"]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem, att_syntax));
        }
    }
}
