SECTIONS {
	/* Define that the text section should be loaded at 0x7C00. */
    .text 0x7c00 : {}
    /* Place build-id after text section. */
    .gnu_build_id : {
      *(.note.gnu.build-id)
    }
    /* Place Boot Signature at the last two bytes of the sector. */
    .signature : AT(ADDR(.text) + 510) {
        SHORT(0xAA55);
    }
}