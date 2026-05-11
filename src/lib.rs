#![no_std]

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

pub const GDT_ENTRIES: usize = 3;

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

// Layout required by LGDT/LIDT: 2-byte limit followed by 4-byte base address.
#[repr(C, packed)]
pub struct TablePointer {
    pub limit: u16,
    pub base: u32,
}
