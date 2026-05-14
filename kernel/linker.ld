ENTRY(kernel_entry)

SECTIONS {
    . = 0;
    .text : { *(.text*) }
    .rodata : { *(.rodata*) }
    .data : { *(.data*) }
    .bss (NOLOAD) : { *(.bss*) }
    /DISCARD/ : { *(.eh_frame*) *(.note*) }
}
