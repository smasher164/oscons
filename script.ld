SECTIONS {
    /* Define that the text section should be loaded at 0x7C00. */
    .text 0x7C00 : {}
    /* Place Boot Signature at the last two bytes of the sector. */
    . : AT(ADDR(.text) + 510) {
        SHORT(0xAA55);
    }
}