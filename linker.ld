ENTRY(_start)

SECTIONS {
    . = 0x7C00;

    .text : {
        KEEP(*(.text.entry))
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note*)
    }

    . = 0x7C00 + 510;
    .boot_sig : {
        SHORT(0xAA55)
    }
}
