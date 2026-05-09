#![no_std]
#![no_main]
// global_asm! has no options(att_syntax); the .att_syntax directive is the
// only way to use AT&T syntax, which Rust flags as "bad_asm_style".
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
    main = sym stage16_main,
);

// Segment:offset pointer for 16-bit real-mode memory access. Used to reach
// physical addresses that require a non-zero segment base, since LLVM's
// 16-bit target truncates flat pointers > 0xFFFF.
#[derive(Clone, Copy)]
struct SegPtr(u16, u16); // (segment, offset)

impl SegPtr {
    #[inline(always)]
    unsafe fn read(self) -> u8 {
        let val: u8;
        asm!(
            "pushw %ds",
            "movw {seg:x}, %ds",
            "movb %ds:({off}), {val}",
            "popw %ds",
            seg = in(reg) self.0,
            off = in(reg) self.1 as u32,
            val = out(reg_byte) val,
            options(att_syntax),
        );
        val
    }

    #[inline(always)]
    unsafe fn write(self, val: u8) {
        asm!(
            "pushw %ds",
            "movw {seg:x}, %ds",
            "movb {val}, %ds:({off})",
            "popw %ds",
            seg = in(reg) self.0,
            off = in(reg) self.1 as u32,
            val = in(reg_byte) val,
            options(att_syntax),
        );
    }
}

#[repr(C, packed)]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    limit_flags: u8, // G, DB, L, AVL in [7:4]; limit[19:16] in [3:0]
    base_high: u8,
}

#[repr(C, packed)]
struct Gdt {
    null: GdtEntry,
    code: GdtEntry,
    data: GdtEntry,
}

// Layout required by LGDT/LIDT: 2-byte limit followed by 4-byte base address.
#[repr(C, packed)]
struct TablePointer {
    limit: u16,
    base: u32,
}

static GDT: Gdt = Gdt {
    null: GdtEntry {
        limit_low: 0,
        base_low: 0,
        base_mid: 0,
        access: 0,
        limit_flags: 0,
        base_high: 0,
    },
    code: GdtEntry {
        limit_low: 0xFFFF,
        base_low: 0x0000,
        base_mid: 0x00,
        access: 0x9A,      // P=1, DPL=0, S=1, exec/read
        limit_flags: 0xCF, // G=1, DB=1 (32-bit), limit[19:16]=0xF
        base_high: 0x00,
    },
    data: GdtEntry {
        limit_low: 0xFFFF,
        base_low: 0x0000,
        base_mid: 0x00,
        access: 0x92, // P=1, DPL=0, S=1, read/write
        limit_flags: 0xCF,
        base_high: 0x00,
    },
};

static EMPTY_IDT: TablePointer = TablePointer { limit: 0, base: 0 };

// Defined in stage32/src/lib.rs; the linker resolves this across the two
// staticlib archives.
extern "C" {
    fn stage32_entry() -> !;
}

#[no_mangle]
fn stage16_main() -> ! {
    bios_interrupt::<0x10>(0x0003); // set video mode 3 (80x25 16-color text)
    if !enable_a20() {
        print(b"Failed to Enter Protected Mode.");
        panic!();
    }
    enter_protected_mode()
}

fn enable_a20() -> bool {
    if check_a20() {
        return true;
    }

    bios_interrupt::<0x15>(0x2401); // BIOS: enable A20
    if check_a20() {
        return true;
    }

    // 8042 keyboard controller method.
    wait_kbc();
    outb(0x64, 0xD1); // write command: write to output port
    wait_kbc();
    outb(0x60, 0xDF); // output port value with A20 bit set
    wait_kbc();
    if check_a20() {
        return true;
    }

    // Fast A20 via System Control Port A. Done last because it can crash
    // on some hardware.
    outb(0x92, inb(0x92) | 0x02);
    check_a20()
}

// Check whether the A20 line is enabled by probing whether physical addresses
// 0x000500 and 0x100500 are distinct. These two addresses are always exactly
// 1 MiB apart, so when A20 is disabled bit 20 of the higher address is forced
// to 0, aliasing both to the same location.
//
// DS=0x0000 → DS:0x0500 = 0x000500; DS=0xFFFF → DS:0x0510 = 0x100500.
fn check_a20() -> bool {
    let p0 = SegPtr(0x0000, 0x0500);
    let p1 = SegPtr(0xFFFF, 0x0510);
    unsafe {
        let saved0 = p0.read();
        let saved1 = p1.read();
        p0.write(0x00);
        p1.write(0xFF);
        let enabled = p0.read() != 0xFF;
        p0.write(saved0);
        p1.write(saved1);
        enabled
    }
}

fn wait_kbc() {
    while inb(0x64) & 0x02 != 0 {}
}

fn enter_protected_mode() -> ! {
    let gdt_ptr = TablePointer {
        limit: (core::mem::size_of_val(&GDT) - 1) as u16,
        base: core::ptr::addr_of!(GDT) as u32,
    };
    lgdt(core::ptr::addr_of!(gdt_ptr));
    lidt(core::ptr::addr_of!(EMPTY_IDT));
    unsafe {
        asm!(
            "cli",
            "movl %cr0, %eax",
            "orl $1, %eax",
            "movl %eax, %cr0",
            "ljmpl $0x8, ${target}", // far jump: atomically sets CS=0x8 (code segment) and jumps
            target = sym stage32_entry,
            options(nostack, noreturn, att_syntax),
        );
    }
}

fn lgdt(ptr: *const TablePointer) {
    unsafe {
        asm!("lgdt ({0})", in(reg) ptr, options(nostack, att_syntax));
    }
}

fn lidt(ptr: *const TablePointer) {
    unsafe {
        asm!("lidt ({0})", in(reg) ptr, options(nostack, att_syntax));
    }
}

fn print(s: &[u8]) {
    for &byte in s {
        bios_interrupt::<0x10>(0x0E00 | byte as u16); // AH=0x0E (TTY output)
    }
}

// Execute BIOS interrupt VECTOR with AX=ax. The const generic ensures the
// vector is a compile-time immediate, as required by the int instruction.
fn bios_interrupt<const VECTOR: u8>(ax: u16) {
    unsafe {
        asm!("int ${0}", const VECTOR, in("ax") ax, options(nostack, nomem, att_syntax));
    }
}

fn outb(port: u16, val: u8) {
    unsafe {
        asm!("outb %al, %dx", in("dx") port, in("al") val, options(nostack, nomem, att_syntax));
    }
}

fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("inb %dx, %al", out("al") val, in("dx") port, options(nostack, nomem, att_syntax));
    }
    val
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem, att_syntax));
        }
    }
}
