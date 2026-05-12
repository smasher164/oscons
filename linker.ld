ENTRY(_start)

SECTIONS {
    . = 0x7C00;
    .stage1 : {
        KEEP(*(.text.entry))
        *(.text.stage1*)
        *(.rodata.stage1*)
        *(.data.stage1*)
    }
    /DISCARD/ : { *(.eh_frame*) *(.note*) *(.bss*) }
    . = 0x7C00 + 510;
    .boot_sig : { SHORT(0xAA55) }
    . = 0x7E00;
    .stage2 : {
        KEEP(*(.text.entry2))
        *(.text* .rodata* .data*)
    }
}
