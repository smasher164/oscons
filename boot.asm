;
; BIOS-based bootloader program for x86 that tries to enter protected mode. On
; success, it prints "Successfully Entered Protected Mode," and on failure, it
; prints "Failed to Enter Protected Mode." This file should be assembled with
; YASM, available at https://yasm.tortall.net.

    bits         16                ; Tell YASM these are 16-bit instructions

global _start                      ; Export the _start symbol
_start:

; Set Video Mode to support 80x25 16 color text (AH=0,AL=3). Do this in advance,
; to enable drawing in protected mode as well.

    mov          ax, 0x3
    int          0x10

; Load the Global Descriptor Table (GDT). See the gdt and GDT_ADDR labels below
; for details on its layout. The GDT contains information needed for the CPU to
; translate a segment:offset into a physical address, and determine the proper
; permissions and attributes for a region of memory.

    lgdt         [GDT_ADDR]

; Try enabling the A20 line. If it fails, print an error message and exit
; bootloader.

    call         enableA20
    cmp          ax, 0
    je           failedA20

; It is now possible to jump into protected mode. Disable interrupts, and
; set the protected mode bit on the special CPU register CR0.

    cli
    mov          eax, cr0
    or           eax,0x1
    mov          cr0, eax

; Jump to the code segment defined in the GDT.
; Selector = 1, Segment = Selector * 8 = 0x8. Offset = protectedMode.

    jmp          0x8:protectedMode 

; Print the failure string and exit bootloader when A20 could not be enabled.
failedA20:
    mov          si, FAILED_STRING ; si = FAILED_STRING
    mov          ah, 0x0E          ; set the interrupt handler to TTY printer
.loopFailedA20:
    lodsb                          ; al = *FAILED_STRING; FAILED_STRING++.
    cmp          al, 0
    je           endFailedA20      ; if (al == 0) goto endFailedA20
    int          0x10              ; execute the interrupt for video services
    jmp          .loopFailedA20
endFailedA20:
    hlt

; Entered Protected Mode. Print SUCCESS_STRING and exit bootloader.
protectedMode:
    bits         32                ; Tell YASM these are 32-bit instructions
    mov          esi, SUCCESS_STRING
    mov          ebx, 0xB8000      ; Set pointer to video memory (color).
.loopSuccess
    lodsb
    cmp          al, 0
    je           end
    or           ah,0xF            ; Set attribute color to white.
    mov          [ebx], ax         ; Write attribute+character to video memory.
    add          ebx, 2            ; Move to next position.
    jmp          .loopSuccess
end:
    hlt

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
SUCCESS_STRING: db "Successfully Entered Protected Mode.", 0