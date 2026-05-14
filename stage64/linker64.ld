ENTRY(stage64_entry)

SECTIONS {
    . = STAGE64_BASE;
    .text.entry64 : { KEEP(*(.text.entry64)) }
    .text : { *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    /DISCARD/ : { *(.eh_frame*) *(.note*) *(.bss*) }
    boot_end = .;
    /* kernel_end is where stage64 copies PT_LOAD segments to. Aligning it
       to 8 makes PT_DYNAMIC and .rela.dyn (each 8-aligned within the kernel
       image) land on 8-byte boundaries in destination memory, allowing slice
       iteration over their entries. No section follows, so no bytes are
       emitted into stage64.bin — the gap is just memory that gets overwritten
       by the segment copy. */
    . = boot_end + KERNEL_SIZE;
    . = ALIGN(8);
    kernel_end = .;
}
