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

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
struct EntryType(u32);

impl EntryType {
    const USABLE: Self = Self(1);
    const RESERVED: Self = Self(2);
    const ACPI_RECLAIMABLE: Self = Self(3);
    const ACPI_NVS: Self = Self(4);
    const BAD_MEMORY: Self = Self(5);
}

impl core::fmt::Display for EntryType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            EntryType::USABLE => write!(f, "Usable"),
            EntryType::RESERVED => write!(f, "Reserved"),
            EntryType::ACPI_RECLAIMABLE => write!(f, "ACPI Reclaimable"),
            EntryType::ACPI_NVS => write!(f, "ACPI NVS"),
            EntryType::BAD_MEMORY => write!(f, "Bad Memory"),
            EntryType(v) => write!(f, "Unknown({v})"),
        }
    }
}

// Each E820 entry describes one region of the physical address space.
#[derive(Clone, Copy)]
#[repr(C)]
struct E820Entry {
    base: u64,
    len: u64,
    entry_type: EntryType,
    // ACPI extended attributes; initialized to 1 (valid) in case BIOS writes
    // only 20 bytes.
    acpi: u32,
}

impl E820Entry {
    const ZERO: Self = Self {
        base: 0,
        len: 0,
        entry_type: EntryType(0),
        acpi: 0,
    };
}

impl core::fmt::Display for E820Entry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let valid = self.acpi & 1 != 0;
        let non_volatile = self.acpi & 2 != 0;
        write!(f, "base: {:x}  len: {:x}  type: {}  acpi: {:#x} (", self.base, self.len, self.entry_type, self.acpi)?;
        if valid {
            if non_volatile {
                write!(f, "valid, non-volatile")?;
            } else {
                write!(f, "valid")?;
            }
        } else {
            write!(f, "invalid")?;
        }
        write!(f, ")")
    }
}

struct MemoryMap {
    entries: [E820Entry; MAX_E820_ENTRIES],
    count: usize,
}

impl MemoryMap {
    const ZERO: Self = Self {
        entries: [E820Entry::ZERO; MAX_E820_ENTRIES],
        count: 0,
    };
}

const SMAP: u32 = 0x534D4150;
const MAX_E820_ENTRIES: usize = 32;
const STAGE2_SECTORS: u8 = 7;

static mut MEMORY_MAP: MemoryMap = MemoryMap::ZERO;

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
            inlateout("al") STAGE2_SECTORS => sectors_read, // al returns count read
            in("ch") 0u8,                        // cylinder 0
            in("cl") 2u8,                        // sector 2
            in("dh") 0u8,                        // head 0
            in("dl") drive,                      // dl already has drive number
            in("bx") 0x7E00u16,                  // load into memory at 0x7e00
            options(att_syntax),
        );
    }
    no_error != 0 && sectors_read == STAGE2_SECTORS
}

#[no_mangle]
fn stage2_main() -> ! {
    let map = unsafe { &mut *(&raw mut MEMORY_MAP) };
    query_memory_map(map);
    for entry in &map.entries[..map.count] {
        let _ = write!(Tty, "{entry}\r\n");
    }
    panic!()
}

fn query_memory_map(map: &mut MemoryMap) {
    let mut continuation = 0u32;
    // ES=0 is set in stage2_entry; no BIOS calls have been made before this
    // that could modify it.
    while map.count < MAX_E820_ENTRIES {
        match query_entry(&mut map.entries[map.count], continuation) {
            None => break,
            Some(next) => {
                map.count += 1;
                if next == 0 { break; }
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
