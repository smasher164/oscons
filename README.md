# OS Construction

This branch contains a BIOS-based bootloader for x86 that prints "Hello, World!" to the screen. YASM is used to assemble the source, and is available at https://yasm.tortall.net. ld is used to link the binary, and objcopy is used to strip its ELF headers. These commands can either be found in GNU Binutils at https://www.gnu.org/software/binutils/ or LLVM at https://lld.llvm.org/ and https://llvm.org/docs/CommandGuide/llvm-objcopy.html. QEMU is used to emulate the hardware and BIOS necessary to run the bootloader, and is available at https://www.qemu.org/. My system runs `lld` so the Unix linker is installed as `ld.lld`.

QEMU can be used to execute the flat binary file by treating it as a disk. Either qemu-system-i386 or qemu-system-x86_64 can be used to run the disk image. The following commands can be used to build and run the bootloader in QEMU:

```
$ yasm -f elf boot.asm -o boot.o
$ ld.lld boot.o -N -T script.ld -b binary -o boot.img
$ objcopy boot.img -S -O binary
$ qemu-system-<i386|x86_64> -drive format=raw,file=boot.img
```

Which should result in output like the following:
![screenshot](https://user-images.githubusercontent.com/12636891/66261025-960b2680-e794-11e9-8982-1b473261ed10.png)