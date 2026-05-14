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

// A single entry in the PT_DYNAMIC array. The dynamic section is a
// null-terminated list of these key/value pairs; we walk it to discover
// where the relocation table lives.
#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Dyn {
    d_tag: i64,
    d_val: u64,
}

// A single relocation entry. The kernel is linked as ET_DYN with base 0, so
// r_offset and r_addend are offsets into the loaded image; r_info packs the
// reloc type into its low 32 bits (and a symbol index into the high 32, which
// is 0 for R_X86_64_RELATIVE since no symbol is involved).
#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Rela {
    r_offset: u64,
    r_info: u64,
    r_addend: i64,
}

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
extern "C" fn stage64_main() -> ! {
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

    // Upper-half canonical virtual base for PML4[k]: bits 63:48 must
    // sign-extend bit 47 (=1 for upper half). Adding expand_base (the PA
    // start of the loaded image) gives the VA the kernel will see, since
    // PML4[k] reuses the identity map's PDPT.
    let virt_base: u64 = 0xFFFF_0000_0000_0000 | (k << 39);
    let kernel_virt_base = virt_base + expand_base as u64;

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
        // While iterating, also capture PT_DYNAMIC so we can apply relocations
        // below without walking the program headers a second time.
        let mut dyn_phdr = None;
        if let Some(phdrs) = elf.segments() {
            for phdr in phdrs {
                match phdr.p_type {
                    elf::abi::PT_LOAD => {
                        let src = elf_ptr.add(phdr.p_offset as usize);
                        let dst = expand_base.add(phdr.p_vaddr as usize);
                        core::ptr::copy_nonoverlapping(src, dst, phdr.p_filesz as usize);
                        core::ptr::write_bytes(
                            dst.add(phdr.p_filesz as usize),
                            0,
                            (phdr.p_memsz - phdr.p_filesz) as usize,
                        );
                    }
                    elf::abi::PT_DYNAMIC => dyn_phdr = Some(phdr),
                    _ => {}
                }
            }
        }

        // The kernel was linked as ET_DYN assuming base 0, so every absolute
        // pointer baked into .data or .rodata currently holds a small offset
        // (its link-time address). The linker recorded each one in .rela.dyn
        // with the rule:
        //
        //     *(load_base + r_offset) = load_base + r_addend
        //
        // RIP-relative code (calls, &FOO in instructions) doesn't appear here
        // — the linker resolved it at link time, since source and target slide
        // together. Only baked-in absolute addresses need fixup.
        //
        // The location to patch lives in our physical memory (expand_base +
        // r_offset), but the value we write is the virtual address the kernel
        // will see once it's running through the upper-half mapping
        // (kernel_virt_base + r_addend).
        if let Some(phdr) = dyn_phdr {
            // The PT_DYNAMIC array tells us where to look for the relocation
            // table. It's a list of {tag, value} pairs terminated by DT_NULL;
            // we only care about three of them — every other tag describes
            // things like shared-library deps or symbol tables that this
            // freestanding kernel doesn't have.
            let dyn_entries = core::slice::from_raw_parts(
                expand_base.add(phdr.p_vaddr as usize) as *const Elf64Dyn,
                phdr.p_memsz as usize / core::mem::size_of::<Elf64Dyn>(),
            );
            let mut rela_vaddr: u64 = 0;
            let mut rela_size: u64 = 0;
            let mut rela_entsize: u64 = core::mem::size_of::<Elf64Rela>() as u64;
            for entry in dyn_entries {
                match entry.d_tag {
                    elf::abi::DT_NULL => break,
                    // DT_RELA: image-relative offset to the relocation table.
                    elf::abi::DT_RELA => rela_vaddr = entry.d_val,
                    // DT_RELASZ: total size of the relocation table in bytes.
                    elf::abi::DT_RELASZ => rela_size = entry.d_val,
                    // DT_RELAENT: size of one entry (always 24 for Elf64_Rela,
                    // but read it rather than assume).
                    elf::abi::DT_RELAENT => rela_entsize = entry.d_val,
                    _ => {}
                }
            }

            // Walk the relocation table. r_info's low 32 bits are the reloc
            // type; we only handle R_X86_64_RELATIVE (load-base + addend).
            // Anything else — a symbolic reloc, TLS, GOT/PLT — means the
            // linker emitted something we don't know how to resolve, which
            // would silently corrupt the kernel; panic instead.
            let relocations = core::slice::from_raw_parts(
                expand_base.add(rela_vaddr as usize) as *const Elf64Rela,
                (rela_size / rela_entsize) as usize,
            );
            for reloc in relocations {
                let r_type = (reloc.r_info & 0xffff_ffff) as u32;
                if r_type != elf::abi::R_X86_64_RELATIVE {
                    let _ = writeln!(vga, "Unsupported reloc type {r_type}.");
                    panic!();
                }
                // expand_base + r_offset: physical address of the pointer
                // field to patch. kernel_virt_base + r_addend: the upper-half
                // virtual address the kernel will see when it dereferences it.
                let dst = expand_base.add(reloc.r_offset as usize) as *mut u64;
                dst.write(kernel_virt_base + reloc.r_addend as u64);
            }
        }

        elf.ehdr.e_entry
    };

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
