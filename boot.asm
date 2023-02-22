;
; BIOS-based bootloader program for x86 that tries to enter protected mode. On
; success, it prints "Successfully Entered Protected Mode," and on failure, it
; prints "Failed to Enter Protected Mode." This file should be assembled with
; YASM, available at https://yasm.tortall.net.

    bits         16                ; Tell YASM these are 16-bit instructions
    section      .text

global _start                      ; Export the _start symbol
_start:
    cli                            ; Disable interrupts

    xor          ax, ax            ; Clear ax
    mov          ds, ax
    mov          es, ax
    mov          ss, ax
    mov          sp, 0x7C00        ; Set stack pointer to top of stack

; Set Video Mode to support 80x25 16 color text (AH=0,AL=3). Do this in advance,
; to enable drawing in protected mode as well.

    mov          ax, 0x3
    int          0x10

; Load 2nd stage bootloader into memory. That's 24 sectors (12K) to the address 0x7e00.

    call         read

; Load the Global Descriptor Table (GDT). See the gdt and GDT_ADDR labels below
; for details on its layout. The GDT contains information needed for the CPU to
; translate a segment:offset into a physical address, and determine the proper
; permissions and attributes for a region of memory.

    lgdt         [GDT_ADDR]

; Try enabling the A20 line. If it fails, print an error message and spin forever.

    call         enableA20
    cmp          ax, 0
    jne          a20Enabled
    mov          si, FAILED_STRING
    jmp          printError

; It is now possible to jump into protected mode. Set the
; protected mode bit on the special CPU register CR0.

a20Enabled:
    mov          eax, cr0
    or           eax, 0x1
    mov          cr0, eax

; Jump to the code segment defined in the GDT.
; Selector = 1, Segment = Selector * 8 = 0x8. Offset = protectedMode.

    jmp          0x8:protectedMode 

read:
    pusha
    ; try reading up to 3 times
    mov          cx, 3
.loop:
    ; if we've tried 3 times, print an error message and exit bootloader
    cmp          cx, 0
    je           .error
    call         tryRead
    cmp          ax, 0
    jne          .success
    dec          cx
    jmp          .loop
.error:
    mov          si, READ_ERROR_STRING
    jmp          printError
.success:
    popa
    ret

tryRead:
    pusha
    mov          ah, 0x2           ; read
    mov          al, 0x18          ; 24 sectors for 12k
    mov          ch, 0             ; starting at cylinder 0
    mov          cl, 0x02          ; starting at sector 2
    mov          dh, 0             ; starting at head 0
    mov          bx, 0x7e00        ; load into memory at 0x7e00
                                   ; dl already has drive number

    int          0x13              ; fire interrupt
    jc           .error            ; if carry flag is set, error
    cmp          al, 0x18          ; check if we read 24 sectors
    jne          .error
    popa
    mov          ax, 1             ; success
    ret
.error:
    popa
    mov          ax, 0
    ret

; Print the failure string and spin forever.
printError:
    mov          ah, 0x0E          ; set the interrupt handler to TTY printer
.loop:
    lodsb                          ; al = *si; si++.
    cmp          al, 0
    je           .end              ; if (al == 0) goto .end
    int          0x10              ; execute the interrupt for video services
    jmp          .loop
.end:
    jmp $


; Enable the A20 line. This removes the restriction of only addressing at most
; 1MB and enables the ability to address up to 4GB. The reason for this
; restriction is historical and purely for backwards compatibility, as engineers
; depended on the wrap-around behavior from accessing past 1MB. The way this is
; done varies between processors, so for maximum compatibility, try multiple
; methods, and check after each one.
; Note: An even better implementation of this function would retry the keyboard
; controller method in a timeout.
enableA20:
    call         checkA20          ; Do an initial check that A20 is enabled.
    cmp          ax, 0
    jne          enabledA20

    mov          ax, 0x2401        ; BIOS interrupt 0x15 with AH=0x24,AL=01.
    int          0x15
    call         checkA20
    cmp          ax, 0
    jne          enabledA20

    call         waitIO            ; Use 8042 Keyboard Controller
    mov          al, 0xD1
    out          0x64, al          ; Send write command to output port.
    call         waitIO
    mov          al, 0xDF
    out          0x60, al          ; Enable A20
    call         checkA20
    cmp          ax, 0
    jne          enabledA20

    in           al, 0x92          ; Read byte from System Control Port A. This
                                   ; method is done last because it may crash.
    or           al, 2             ; Set Bit 1.
    out          0x92, al          ; Write back.
    call         checkA20
enabledA20:
    ret

; Wait until the I/O port of the keyboard controller is not busy.
waitIO:
    in           al, 0x64
    test         al, 0x2
    jnz          waitIO
    ret

; Check that the A20 line has been enabled. It works by writing a 0x00 byte to
; 0x0000:0x0500 and 0xFF byte to 0xFFFF:0x0510. If the byte at 0x0000:0x0500 is
; 0xFF, then the access wraps around, and the A20 line has not been enabled. The
; values at these addresses are saved and restored to allow this function to be
; re-run.
checkA20:
    push         ds
    push         es
    mov          ax, 0xFFFF
    mov          ds, ax
    not          ax
    mov          es, ax
    mov          cx, word [es:0x0500]
    mov          dx, word [ds:0x0510]
    mov          byte [es:0x0500], 0x00
    mov          byte [ds:0x0510], 0xFF
    cmp          byte [es:0x0500], 0xFF
    mov          word [es:0x0500], cx
    mov          word [ds:0x0510], dx
    je           checkA20__exit
    mov          ax, 1
checkA20__exit:
    pop          es
    pop          ds
    ret

; Entered Protected Mode.
protectedMode:
    bits         32                ; Tell YASM these are 32-bit instructions
    ; Set up the stack
    mov          eax, 0x10         ; The data segment selector is 0x10.
    mov          ds, eax
    mov          es, eax
    mov          fs, eax
    mov          gs, eax
    mov          ss, eax
    mov          esp, stack_end    ; Set the stack pointer (stack grows down).
    jmp          0x7e00            ; Jump to the second sector.

; Each definition in the GDT is an 8-byte descriptor. They are respectively, a
; NULL descriptor, code segment descriptor, and data segment descriptor.
gdt:

;   NULL Descriptor
;  ┌──────┬───────┬───────┬────────┬──────┬───────┐
;  │ Base │ Flags │ Limit │ Access │ Base │ Limit │
;  ├──────┼───────┼───────┼────────┼──────┼───────┤
;  │ 0x0  │ 0x0   │ 0x0   │ 0x0    │ 0x0  │ 0x0   │
;  └──────┴───────┴───────┴────────┴──────┴───────┘
    dq 0

;   Code Segment Descriptor                           Flags
;  ┌──────┬───────┬───────┬────────┬──────┬────────┐ ┌─────────────┬──────┬──────┬───────┐
;  │ Base │ Flags │ Limit │ Access │ Base │ Limit  │ │ Granularity │ Size │ Long │ Extra │
;  ├──────┼───────┼───────┼────────┼──────┼────────┤ ├─────────────┼──────┼──────┼───────┤
;  │ 0x0  │ 0xB   │ 0xF   │ 0x9A   │ 0x0  │ 0xFFFF │ │ 0b1         │ 0b1  │ 0b0  │ 0b0   │
;  └──────┴───────┴───────┴────────┴──────┴────────┘ └─────────────┴──────┴──────┴───────┘
;   Access
;  ┌─────────┬───────────┬──────┬────────────┬───────────┬────────────┬──────────┐
;  │ Present │ Privilege │ Type │ Executable │ Direction │ Read/Write │ Accessed │
;  ├─────────┼───────────┼──────┼────────────┼───────────┼────────────┼──────────┤
;  │ 0b1     │ 0b00      │ 0b1  │ 0b1        │ 0b0       │ 0b1        │ 0b0      │
;  └─────────┴───────────┴──────┴────────────┴───────────┴────────────┴──────────┘
    dq 0x00CF9A000000FFFF

;   Data Segment Descriptor                           Flags
;  ┌──────┬───────┬───────┬────────┬──────┬────────┐ ┌─────────────┬──────┬──────┬───────┐
;  │ Base │ Flags │ Limit │ Access │ Base │ Limit  │ │ Granularity │ Size │ Long │ Extra │
;  ├──────┼───────┼───────┼────────┼──────┼────────┤ ├─────────────┼──────┼──────┼───────┤
;  │ 0x0  │ 0xB   │ 0xF   │ 0x92   │ 0x0  │ 0xFFFF │ │ 0b1         │ 0b1  │ 0b0  │ 0b0   │
;  └──────┴───────┴───────┴────────┴──────┴────────┘ └─────────────┴──────┴──────┴───────┘
;   Access
;  ┌─────────┬───────────┬──────┬────────────┬───────────┬────────────┬──────────┐
;  │ Present │ Privilege │ Type │ Executable │ Direction │ Read/Write │ Accessed │
;  ├─────────┼───────────┼──────┼────────────┼───────────┼────────────┼──────────┤
;  │ 0b1     │ 0b00      │ 0b1  │ 0b0        │ 0b0       │ 0b1        │ 0b0      │
;  └─────────┴───────────┴──────┴────────────┴───────────┴────────────┴──────────┘
    dq 0x00CF92000000FFFF

; Define pointer structure to the GDT defined above.
GDT_ADDR:
    dw           24-1              ; Limit = size of GDT - 1
    dd           gdt               ; Address of gdt

; Declare NULL-terminated string constants.

FAILED_STRING: db "Failed to Enter Protected Mode.", 0
READ_ERROR_STRING: db "Error reading from disk.", 0

; Zero out the remaining of the 512 bytes and
; set the last two bytes to the signature.

    times 510 - ($-$$) db 0
    dw 0xAA55

; At address 0x7e00, the second sector of the disk is loaded.
; Print SUCCESS_STRING and spin forever.
    mov          esi, SUCCESS_STRING
    mov          ebx, 0xB8000      ; Set pointer to video memory (color).
.loop:
    lodsb
    cmp          al, 0
    je           .end
    or           ah,0xF            ; Set attribute color to white.
    mov          [ebx], ax         ; Write attribute+character to video memory.
    add          ebx, 2            ; Move to next position.
    jmp          .loop
.end:
    jmp $

SUCCESS_STRING: db "Successfully Entered Protected Mode Outside MBR.", 0

; Declare the start and end of the stack in the .bss section.
    section .bss

stack_begin:
    resb 4096                      ; Reserve 4 KiB of stack space
stack_end: