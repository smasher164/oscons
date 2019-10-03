bits 16
org 0x7c00
mov ax, 0x3
int 0x10
lgdt [GDT_ADDR]

call enableA20
cmp ax, 0
mov si, FAILED_STRING
mov ah, 0x0E
je loopFailedA20

mov eax, cr0
or eax,0x1 ; set the protected mode bit on special CPU reg cr0
mov cr0, eax
jmp 0x8:protectedMode

enableA20:
    call checkA20
    cmp ax, 0
    jne enabledA20

    mov ax, 0x2401 ; BIOS
    int 0x15
    call checkA20
    cmp ax, 0
    jne enabledA20

    call waitIO  ; keyboard
    mov al, 0xD1
    out 0x64, al ; send write command to output port
    call waitIO
    mov al, 0xdf
    out 0x60, al ; enable A20
    call checkA20
    cmp ax, 0
    jne enabledA20

    in al, 0x92 ; fast
    or al, 2
    out 0x92, al
    call checkA20
enabledA20:
    ret

waitIO:
    in al, 0x64
    test al, 0x2
    jnz waitIO
    ret

checkA20:
    push ds
    push es
    mov ax, 0xFFFF
    mov ds, ax
    not ax
    mov es, ax
    mov cx, word [es:0x0500]
    mov dx, word [ds:0x0510]
    mov byte [es:0x0500], 0x00
    mov byte [ds:0x0510], 0xFF
    cmp byte [es:0x0500], 0xFF
    mov word [es:0x0500], cx
    mov word [ds:0x0510], dx
    je checkA20__exit
    mov ax, 1
checkA20__exit:
    pop es
    pop ds
    ret

loopFailedA20:
    lodsb
    cmp al, 0
    je endFailedA20
    int 0x10
    jmp loopFailedA20
endFailedA20:
    hlt

gdt:
    dq 0
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
GDT_ADDR:   
    dw 24
    dd gdt

protectedMode:
bits 32
mov esi, SUCCESS_STRING
mov ebx, 0xB8000
.loopSuccess
    lodsb
    cmp al, 0
    je end
    or eax,0xF00
    mov [ebx], ax
    add ebx, 2
    jmp .loopSuccess
end:
    cli
    hlt

bits 16
FAILED_STRING: db "Failed to Boot!", 0   ; Declare NULL-terminated string constant
SUCCESS_STRING: db "Entered Protected Mode!", 0   ; Declare NULL-terminated string constant
    times 510 - ($-$$) db 0           ; Zero out the remaining of the 512 bytes
    dw           0xAA55               ; Set last two bytes to signature