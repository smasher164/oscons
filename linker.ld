ENTRY(_start)

SECTIONS {
    . = 0x7C00;
    .stage1 : {
        KEEP(*libstage16.a:*(.text.entry))
        *libstage16.a:*(.text.stage1*)
        *libstage16.a:*(.rodata.stage1*)
        *libstage16.a:*(.data.stage1*)
    }
    /DISCARD/ : { *(.eh_frame*) *(.note*) }
    . = 0x7C00 + 510;
    .boot_sig : { SHORT(0xAA55) }
    . = 0x7E00;
    .stage2 : {
        KEEP(*libstage16.a:*(.text.entry2))
        *libstage16.a:*(.text* .rodata* .data*)
        KEEP(*libstage32.a:*(.text.entry32))
        *libstage32.a:*(.text* .rodata* .data*)
    }
}
