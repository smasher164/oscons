#![no_std]

use core::arch::asm;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq)]
pub struct EntryType(pub u32);

impl EntryType {
    pub const USABLE: Self = Self(1);
    pub const RESERVED: Self = Self(2);
    pub const ACPI_RECLAIMABLE: Self = Self(3);
    pub const ACPI_NVS: Self = Self(4);
    pub const BAD_MEMORY: Self = Self(5);
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
pub struct E820Entry {
    pub base: u64,
    pub len: u64,
    pub entry_type: EntryType,
    // ACPI extended attributes; initialized to 1 (valid) in case BIOS writes
    // only 20 bytes.
    pub acpi: u32,
}

impl E820Entry {
    pub const ZERO: Self = Self {
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
        write!(
            f,
            "base: {:x}  len: {:x}  type: {}  acpi: {:#x} (",
            self.base, self.len, self.entry_type, self.acpi
        )?;
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

pub const MAX_E820_ENTRIES: usize = 32;

#[repr(C)]
pub struct MemoryMap {
    pub entries: [E820Entry; MAX_E820_ENTRIES],
    pub count: usize,
}

impl MemoryMap {
    pub const ZERO: Self = Self {
        entries: [E820Entry::ZERO; MAX_E820_ENTRIES],
        count: 0,
    };
}

pub const GDT_ENTRIES: usize = 4;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_mid: u8,
    pub access: u8,
    pub limit_flags: u8, // G, DB, L, AVL in [7:4]; limit[19:16] in [3:0]
    pub base_high: u8,
}

impl GdtEntry {
    pub const ZERO: Self = Self {
        limit_low: 0,
        base_low: 0,
        base_mid: 0,
        access: 0,
        limit_flags: 0,
        base_high: 0,
    };
}

#[repr(C, align(4096))]
pub struct PageTable(pub [u64; 512]);

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_RW: u64 = 1 << 1;
pub const PAGE_PS: u64 = 1 << 7;

pub const VGA_BUF: *mut u16 = 0xB8000 as *mut u16;
pub const VGA_COLS: u32 = 80;
pub const VGA_ROWS: u32 = 25;
pub const WHITE_ON_BLACK: u8 = 0x0F;

pub fn vga_cell(attr: u8, ch: u8) -> u16 {
    (attr as u16) << 8 | ch as u16
}

pub unsafe fn vga_write(row: u32, col: u32, val: u16) {
    VGA_BUF
        .add(row as usize * 80 + col as usize)
        .write_volatile(val);
}

// Row and col are u32 so the layout is identical when shared between 32-bit
// and 64-bit stages.
#[repr(C)]
pub struct Vga {
    pub row: u32,
    pub col: u32,
}

impl Vga {
    pub fn clear(&mut self) {
        for i in 0..VGA_ROWS * VGA_COLS {
            unsafe { vga_write(i / VGA_COLS, i % VGA_COLS, vga_cell(WHITE_ON_BLACK, b' ')) };
        }
        self.row = 0;
        self.col = 0;
    }
}

impl core::fmt::Write for Vga {
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

// Layout required by LGDT/LIDT: 2-byte limit followed by 4-byte base address.
#[repr(C, packed)]
pub struct TablePointer {
    pub limit: u16,
    pub base: u32,
}

// Memory operand layout required by ljmpl: offset first, then selector.
#[repr(C, packed)]
pub struct FarPtr {
    pub offset: u32,
    pub selector: u16,
}

pub fn far_jump(ptr: FarPtr) -> ! {
    unsafe { asm!("ljmpl *({0})", in(reg) &ptr, options(noreturn, att_syntax)) }
}

pub fn cli() {
    unsafe { asm!("cli", options(nostack, nomem, att_syntax)) }
}

pub fn sti() {
    unsafe { asm!("sti", options(nostack, nomem, att_syntax)) }
}

pub fn lgdt(ptr: *const TablePointer) {
    unsafe { asm!("lgdt ({0})", in(reg) ptr, options(nostack, att_syntax)) }
}

pub fn lidt(ptr: *const TablePointer) {
    unsafe { asm!("lidt ({0})", in(reg) ptr, options(nostack, att_syntax)) }
}

pub fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe {
        asm!("inb %dx, %al", out("al") val, in("dx") port, options(nostack, nomem, att_syntax))
    }
    val
}

pub fn outb(port: u16, val: u8) {
    unsafe {
        asm!("outb %al, %dx", in("dx") port, in("al") val, options(nostack, nomem, att_syntax))
    }
}

pub fn read_cr0() -> usize {
    let val: usize;
    unsafe { asm!("mov %cr0, {0}", out(reg) val, options(nostack, att_syntax)) }
    val
}

pub fn write_cr0(val: usize) {
    unsafe { asm!("mov {0}, %cr0", in(reg) val, options(nostack, att_syntax)) }
}

pub fn read_cr3() -> usize {
    let val: usize;
    unsafe { asm!("mov %cr3, {0}", out(reg) val, options(nostack, att_syntax)) }
    val
}

pub fn write_cr3(val: usize) {
    unsafe { asm!("mov {0}, %cr3", in(reg) val, options(nostack, att_syntax)) }
}

pub fn read_cr4() -> usize {
    let val: usize;
    unsafe { asm!("mov %cr4, {0}", out(reg) val, options(nostack, att_syntax)) }
    val
}

pub fn write_cr4(val: usize) {
    unsafe { asm!("mov {0}, %cr4", in(reg) val, options(nostack, att_syntax)) }
}

pub fn read_msr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi, options(nostack, nomem, att_syntax))
    }
    (hi as u64) << 32 | lo as u64
}

pub fn write_msr(msr: u32, val: u64) {
    let lo = val as u32;
    let hi = (val >> 32) as u32;
    unsafe {
        asm!("wrmsr", in("ecx") msr, in("eax") lo, in("edx") hi, options(nostack, nomem, att_syntax))
    }
}

// Passed from stage64 to the kernel entry point.
#[repr(C)]
pub struct BootInfo {
    pub kernel_virt_base: u64,
    pub memory_map: *const MemoryMap,
    pub pml4: *mut PageTable,
}

// Retries until the hardware RNG has entropy available (CF=1).
#[cfg(target_arch = "x86_64")]
pub fn rdrand64() -> u64 {
    loop {
        let val: u64;
        let ok: u8;
        unsafe {
            asm!(
                "rdrand {val}",
                "setc {ok}",
                val = out(reg) val,
                ok = lateout(reg_byte) ok,
                options(nostack, nomem, att_syntax),
            );
        }
        if ok != 0 {
            return val;
        }
    }
}
