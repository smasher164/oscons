#![no_std]
#![no_main]
// global_asm! has no options(att_syntax); the .att_syntax directive is the
// only way to use AT&T syntax, which Rust flags as "bad_asm_style".
#![allow(bad_asm_style)]

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use elf::endian::LittleEndian;
use elf::ElfBytes;
use oscons::{rdrand64, write_cr3, BootInfo, MemoryMap, PageTable, Vga, PAGE_PRESENT, PAGE_RW};

#[link_section = ".data"]
static mut STACK: [u8; 4096] = [0u8; 4096];

// CS was set to 0x18 by the far jump from stage32; clear the data segment
// registers that stage32 set to 0x10 — they are vestigial in 64-bit mode.
// Set up a fresh stack from STACK[], then call into Rust.
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
    "leaq {stack}(%rip), %rsp",
    "addq $4096, %rsp",
    "call {main}",
    stack = sym STACK,
    main = sym stage64_main,
);

extern "C" {
    static boot_end: u8;
    static kernel_end: u8;
    static mut VGA: Vga;
    static mut PML4: PageTable;
    static PDPT: PageTable;
    static MEMORY_MAP: MemoryMap;
}

#[no_mangle]
fn stage64_main() -> ! {
    #[allow(clippy::deref_addrof)]
    let vga = unsafe { &mut *(&raw mut VGA) };

    // Pick a random PML4 index in the upper half (256–511).
    let k = 256 + (rdrand64() & 0xFF);

    // Add upper-half kernel mapping: PML4[k] → same PDPT as the identity map.
    // PDPT[0] is a 1 GB page covering PA 0–1 GB, which includes boot_end.
    unsafe {
        PML4.0[k as usize] = (&raw const PDPT as u64) | PAGE_PRESENT | PAGE_RW;
    }
    // Reload CR3 to flush the TLB so the new PML4 entry takes effect.
    write_cr3((&raw const PML4) as usize);

    let elf_ptr = &raw const boot_end;
    let expand_base = &raw const kernel_end as *mut u8;
    let kernel_size = expand_base as usize - elf_ptr as usize;

    let e_entry = unsafe {
        let data = core::slice::from_raw_parts(elf_ptr, kernel_size);
        let elf = match ElfBytes::<LittleEndian>::minimal_parse(data) {
            Ok(e) => e,
            Err(_) => {
                let _ = writeln!(vga, "Failed to parse kernel ELF.");
                panic!();
            }
        };

        // Expand each PT_LOAD segment into memory immediately after the ELF
        // file. Source (boot_end..kernel_end) and destination (kernel_end+)
        // are disjoint, so segment descriptors remain readable throughout.
        if let Some(phdrs) = elf.segments() {
            for phdr in phdrs {
                if phdr.p_type != elf::abi::PT_LOAD {
                    continue;
                }
                let src = elf_ptr.add(phdr.p_offset as usize);
                let dst = expand_base.add(phdr.p_vaddr as usize);
                core::ptr::copy_nonoverlapping(src, dst, phdr.p_filesz as usize);
                core::ptr::write_bytes(
                    dst.add(phdr.p_filesz as usize),
                    0,
                    (phdr.p_memsz - phdr.p_filesz) as usize,
                );
            }
        }

        elf.ehdr.e_entry
    };

    // Upper-half canonical virtual base for PML4[k]:
    // bits 63:48 must sign-extend bit 47 (=1 for upper half).
    let virt_base: u64 = 0xFFFF_0000_0000_0000 | (k << 39);
    // The live kernel image starts at kernel_end in physical memory.
    // VA (virt_base + kernel_end) maps to PA kernel_end via the 1 GB page.
    let kernel_virt_base = virt_base + expand_base as u64;
    let entry_virt = kernel_virt_base + e_entry;

    let _ = writeln!(vga, "Jumping to kernel at {:#x}.", entry_virt);

    let info = BootInfo {
        kernel_virt_base,
        memory_map: &raw const MEMORY_MAP,
        pml4: &raw mut PML4,
    };
    // with_exposed_provenance is the blessed integer-to-pointer cast API;
    // the subsequent transmute to a fn pointer is the only available path
    // since Rust has no fn-pointer-from-address API.
    let fn_ptr: *const () = core::ptr::with_exposed_provenance(entry_virt as usize);
    let kernel_entry: extern "C" fn(*const BootInfo) -> ! = unsafe { core::mem::transmute(fn_ptr) };
    kernel_entry(&raw const info);
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nostack, nomem, att_syntax));
        }
    }
}
