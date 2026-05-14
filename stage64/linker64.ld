ENTRY(stage64_entry)

SECTIONS {
    . = STAGE64_BASE;
    .text.entry64 : { KEEP(*(.text.entry64)) }
    .text : { *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    /DISCARD/ : { *(.eh_frame*) *(.note*) *(.bss*) }
    boot_end = .;
    kernel_end = boot_end + KERNEL_SIZE;
}
