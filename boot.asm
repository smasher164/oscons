;
; Simple BIOS-based bootloader program for x86 that prints "Hello, World!" to
; the screen. This file should be assembled with YASM, available at
; https://yasm.tortall.net.

    bits         16                   ; Tell YASM these are 16-bit instructions

global _start                         ; Export the _start symbol
_start:
    xor          ax, ax               ; clear ax
    mov          ds, ax
    mov          es, ax
    mov          ss, ax
    mov          sp, 0x7C00           ; set stack pointer to top of stack

; Call print_hello function.

    mov          si, HELLO_STRING     ; si = HELLO_STRING
    call         print_hello
    hlt

; Loop over and print each character in the string until NULL terminator
; is encountered.

print_hello:
    mov          ah, 0x0E             ; set interrupt handler to TTY printer
.loop:
    lodsb                             ; al = *HELLO_STRING; HELLO_STRING++.
    cmp          al, 0
    je           end                  ; if (al == 0) goto end
    int          0x10                 ; execute the interrupt for video services
    jmp          .loop
end:
    ret                               ; return from function

HELLO_STRING: db "Hello, World!", 0   ; Declare NULL-terminated string constant
