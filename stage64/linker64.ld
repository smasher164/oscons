ENTRY(stage64_entry)

SECTIONS {
    . = STAGE64_BASE;
    .text.entry64 : { KEEP(*(.text.entry64)) }
    .text : { *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    /DISCARD/ : { *(.eh_frame*) *(.note*) *(.bss*) }
}
