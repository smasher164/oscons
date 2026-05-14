ENTRY(kernel_entry)

SECTIONS {
    . = 0;
    .text : { *(.text*) }
    .rodata : { *(.rodata*) }
    .dynsym : { *(.dynsym) }
    .dynstr : { *(.dynstr) }
    .rela.dyn : { *(.rela.dyn) }
    .dynamic : { *(.dynamic) }
    .data : { *(.data*) }
    .bss (NOLOAD) : { *(.bss*) }
    /DISCARD/ : { *(.eh_frame*) *(.note*) *(.gnu.hash) *(.hash) *(.comment) }
}
