#![no_std]
#![no_main]
// global_asm! has no options(att_syntax); the .att_syntax directive is the
// only way to use AT&T syntax, which Rust flags as "bad_asm_style".
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use oscons::MemoryMap;

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

const VGA_BUF: *mut u16 = 0xB8000 as *mut u16;
const WHITE_ON_BLACK: u8 = 0x0F;

fn vga_cell(attr: u8, ch: u8) -> u16 {
    (attr as u16) << 8 | ch as u16
}

unsafe fn vga_write(row: usize, col: usize, val: u16) {
    VGA_BUF.add(row * 80 + col).write_volatile(val);
}

// Filled by stage16 before entering protected mode; accessible here since
// stage16's memory remains in place after the mode switch.
extern "C" {
    static MEMORY_MAP: MemoryMap;
}

struct Vga {
    col: usize,
    row: usize,
}

impl Write for Vga {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            match byte {
                b'\n' => {
                    self.row += 1;
                    self.col = 0;
                }
                b'\r' => {
                    self.col = 0;
                }
                _ => {
                    if self.col >= 80 {
                        self.row += 1;
                        self.col = 0;
                    }
                    unsafe {
                        vga_write(self.row, self.col, vga_cell(WHITE_ON_BLACK, byte));
                    }
                    self.col += 1;
                }
            }
        }
        Ok(())
    }
}

#[no_mangle]
fn stage32_main() -> ! {
    let map = unsafe { &*(&raw const MEMORY_MAP) };
    let mut vga_buf = Vga { col: 0, row: 0 };
    for entry in &map.entries[..map.count] {
        let _ = write!(vga_buf, "{entry}\r\n");
    }
    let _ = write!(vga_buf, "Successfully entered protected mode.");
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
