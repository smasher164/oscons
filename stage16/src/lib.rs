#![no_std]
#![no_main]
// global_asm! has no options(att_syntax); the .att_syntax directive is the
// only way to use AT&T syntax, which Rust flags as "bad_asm_style".
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use oscons::{
    cli, far_jump, inb, lgdt, lidt, outb, read_cr0, write_cr0, E820Entry, FarPtr, GdtEntry,
    MemoryMap, TablePointer, GDT_ENTRIES, MAX_E820_ENTRIES,
};

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
    "call {main}",
    main = sym stage1_main,
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
    "call {main}",
    main = sym stage2_main,
);

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

const SMAP: u32 = 0x534D4150;
const STAGE2_SECTORS: u8 = 48;

static GDT: [GdtEntry; GDT_ENTRIES] = [
    GdtEntry::ZERO, // 0: null
    GdtEntry {
        // 1: 32-bit kernel code
        limit_low: 0xFFFF,
        base_low: 0x0000,
        base_mid: 0x00,
        access: 0x9A,             // P=1, DPL=0, S=1, exec/read
        limit_flags: 0b1100_1111, // G=1, DB=1, L=0, AVL=0, limit=0xF
        base_high: 0x00,
    },
    GdtEntry {
        // 2: kernel data
        limit_low: 0xFFFF,
        base_low: 0x0000,
        base_mid: 0x00,
        access: 0x92,             // P=1, DPL=0, S=1, read/write
        limit_flags: 0b1100_1111, // G=1, DB=1, L=0, AVL=0, limit=0xF
        base_high: 0x00,
    },
    GdtEntry {
        // 3: 64-bit kernel code
        limit_low: 0xFFFF,
        base_low: 0x0000,
        base_mid: 0x00,
        access: 0x9A,             // P=1, DPL=0, S=1, exec/read
        limit_flags: 0b1010_1111, // G=1, DB=0, L=1, AVL=0, limit=0xF
        base_high: 0x00,
    },
];

static EMPTY_IDT: TablePointer = TablePointer { limit: 0, base: 0 };

#[no_mangle]
#[link_section = ".data"]
static mut MEMORY_MAP: MemoryMap = MemoryMap::ZERO;

// BIOS sets DL to the drive number before jumping to 0x7C00. Saved here before
// any function call can clobber it.
#[link_section = ".data.stage1"]
static mut DRIVE: u8 = 0;

// Label defined in global_asm! above; used to jump to stage 2 by symbol rather
// than hardcoding 0x7E00.
extern "C" {
    fn stage2_entry() -> !;
}

// Defined in stage32/src/lib.rs; the linker resolves this across the two
// staticlib archives.
extern "C" {
    fn stage32_entry() -> !;
}

#[no_mangle]
#[link_section = ".text.stage1"]
extern "C" fn stage1_main() -> ! {
    let drive = unsafe { DRIVE };
    for _ in 0..3 {
        if try_disk_read(drive) {
            unsafe { asm!("ljmpw $0, ${0}", sym stage2_entry, options(noreturn, att_syntax)) }
        }
    }
    let _ = Tty.write_str("Error reading from disk.");
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
            in("ah") 2u8,                                    // read sectors
            inlateout("al") STAGE2_SECTORS => sectors_read,  // al returns count read
            in("ch") 0u8,                                    // cylinder 0
            in("cl") 2u8,                                    // sector 2
            in("dh") 0u8,                                    // head 0
            in("dl") drive,                                  // dl already has drive number
            in("bx") 0x7E00u16,                              // load into memory at 0x7e00
            options(att_syntax),
        );
    }
    no_error != 0 && sectors_read == STAGE2_SECTORS
}

#[no_mangle]
extern "C" fn stage2_main() -> ! {
    bios_interrupt::<0x10>(0x0003); // set video mode 3 (80x25 16-color text)
    let map = unsafe { &mut *(&raw mut MEMORY_MAP) };
    query_memory_map(map);
    if !enable_a20() {
        let _ = Tty.write_str("Failed to enable A20.");
        panic!();
    }
    enter_protected_mode()
}

fn query_memory_map(map: &mut MemoryMap) {
    let mut continuation = 0u32;
    while map.count < MAX_E820_ENTRIES {
        match query_entry(&mut map.entries[map.count], continuation) {
            None => break,
            Some(next) => {
                map.count += 1;
                if next == 0 {
                    break;
                }
                continuation = next;
            }
        }
    }
}

// Returns Some(next_continuation) on success, where 0 means this was the last
// entry. Returns None on error.
fn query_entry(entry: &mut E820Entry, continuation: u32) -> Option<u32> {
    entry.acpi = 1;
    let entry_ptr = entry as *mut E820Entry as u32 as u16;
    let mut eax: u32;
    let mut next: u32;
    let success: u8;
    unsafe {
        asm!(
            "xorw %ax, %ax",
            "movw %ax, %es",
            "movl $0xE820, %eax",
            "int $0x15",
            "setnc {ok}",
            ok = lateout(reg_byte) success,
            out("eax") eax,
            inout("ebx") continuation => next,
            in("ecx") 24u32,
            in("edx") SMAP,
            in("di") entry_ptr,
            options(att_syntax),
        );
    }
    if success != 0 && eax == SMAP {
        Some(next)
    } else {
        None
    }
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
        base: (&raw const GDT) as u32,
    };
    lgdt(&raw const gdt_ptr);
    lidt(&raw const EMPTY_IDT);
    cli();
    write_cr0(read_cr0() | 1);
    far_jump(FarPtr {
        selector: 0x8,
        offset: stage32_entry as *const () as u32,
    });
}

fn bios_interrupt<const VECTOR: u8>(ax: u16) {
    unsafe {
        asm!("int ${0}", const VECTOR, in("ax") ax, options(nostack, att_syntax));
    }
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
            options(nostack, att_syntax),
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
