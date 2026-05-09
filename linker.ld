ENTRY(_start)

SECTIONS {
    . = 0x7C00;
    .text : {
        KEEP(*libstage16.a:*(.text.entry))
        *libstage16.a:*(.text* .rodata* .data*)
        KEEP(*libstage32.a:*(.text.entry32))
        *libstage32.a:*(.text* .rodata* .data*)
    }
    /DISCARD/ : { *(.eh_frame*) *(.note*) }
    . = 0x7C00 + 510;
    .boot_sig : { SHORT(0xAA55) }
}
