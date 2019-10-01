;
; Simple BIOS-based bootloader program for x86 that prints "Hello, World!" to
; the screen. This file should be assembled with YASM, available at
; https://yasm.tortall.net.

    bits         16                   ; Tell YASM these are 16-bit instructions
    org          0x7C00               ; Tell YASM to load bootloader at 0x7C00
    mov          si, HELLO_STRING     ; si = HELLO_STRING
    mov          ah, 0x0E             ; set interrupt handler to TTY printer

; Loop over and print each character in the string until NULL terminator
; is encountered.

.loop:
    lodsb                             ; al = *HELLO_STRING; HELLO_STRING++.
    cmp          al, 0
    je           end                  ; if (al == 0) goto end
    int          0x10                 ; execute the interrupt for video services
    jmp          .loop

end:
    hlt                               ; exit bootloader

HELLO_STRING: db "Hello, World!", 0   ; Declare NULL-terminated string constant
    times 510 - ($-$$) db 0           ; Zero out the remaining of the 512 bytes
    dw           0xAA55               ; Set last two bytes to signature
