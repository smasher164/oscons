#![no_std]
#![no_main]
// global_asm! has no options(att_syntax); the .att_syntax directive is the
// only way to use AT&T syntax, which Rust flags as "bad_asm_style".
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use oscons::{
    far_jump, read_cr0, read_cr4, read_msr, write_cr0, write_cr3, write_cr4, write_msr, FarPtr,
    MemoryMap, PageTable, Vga, PAGE_PRESENT, PAGE_PS, PAGE_RW,
};

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

// Filled by stage16 before entering protected mode; accessible here since
// stage16's memory remains in place after the mode switch.
extern "C" {
    static MEMORY_MAP: MemoryMap;
}

// Defined in stage64/src/lib.rs; the linker resolves this across archives.
extern "C" {
    fn stage64_entry() -> !;
}

const EFER_MSR: u32 = 0xC0000080;
const EFER_LME: u64 = 1 << 8;
const CR4_PAE: usize = 1 << 5;
const CR0_PG: usize = 1 << 31;

#[link_section = ".data"]
static mut PML4: PageTable = PageTable([0u64; 512]);
#[link_section = ".data"]
static mut PDPT: PageTable = PageTable([0u64; 512]);

// Preserved across the mode switch so stage64 can continue printing below the
// memory map without needing to know how many rows it occupied.
#[no_mangle]
#[link_section = ".data"]
static mut VGA: Vga = Vga { row: 0, col: 0 };

#[no_mangle]
fn stage32_main() -> ! {
    let map = unsafe { &*(&raw const MEMORY_MAP) };
    let vga = unsafe { &mut *(&raw mut VGA) };
    for entry in &map.entries[..map.count] {
        let _ = write!(vga, "{entry}\r\n");
    }
    enter_long_mode()
}

fn enter_long_mode() -> ! {
    unsafe {
        // PML4[0] points to PDPT, covering the first 512GB of virtual address space.
        PML4.0[0] = (&raw const PDPT as u64) | PAGE_PRESENT | PAGE_RW;
        // PDPT[0] is a 1GB page (PS=1) starting at physical 0, identity-mapping the first 1GB.
        PDPT.0[0] = PAGE_PRESENT | PAGE_RW | PAGE_PS;
    }
    // CR3 holds the physical address of PML4; the CPU walks from here on every address translation.
    write_cr3(&raw const PML4 as usize);
    // PAE must be enabled before long mode; it widens page table entries to 64 bits.
    write_cr4(read_cr4() | CR4_PAE);
    // EFER.LME arms long mode; it activates when paging is enabled below.
    write_msr(EFER_MSR, read_msr(EFER_MSR) | EFER_LME);
    // Setting CR0.PG with EFER.LME set transitions the CPU into long mode (compatibility mode
    // until the far jump below switches CS to the 64-bit code segment).
    write_cr0(read_cr0() | CR0_PG);
    // Selector 0x18 = GDT index 3 (64-bit code segment); the far jump atomically loads CS
    // and completes the transition to 64-bit mode.
    far_jump(FarPtr {
        offset: stage64_entry as *const () as u32,
        selector: 0x18,
    });
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem, att_syntax));
        }
    }
}
