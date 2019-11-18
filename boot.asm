;
; Simple BIOS-based bootloader program for x86 that prints "Hello, World!" to
; the screen. This file should be assembled with YASM, available at
; https://yasm.tortall.net.

    bits         16                   ; Tell YASM these are 16-bit instructions
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