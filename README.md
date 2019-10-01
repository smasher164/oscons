# OS Construction

This branch contains a BIOS-based bootloader for x86 that prints enters protected mode. YASM is used to assemble the source, and is available at https://yasm.tortall.net. QEMU is used to emulate the hardware and BIOS necessary to run the bootloader, and is available at https://www.qemu.org/.

QEMU can be used to execute the flat binary file by treating it as a disk. Either qemu-system-i386 or qemu-system-x86_64 can be used to run the disk image. The following commands can be used to build and run the bootloader in QEMU:

```
$ yasm -f bin boot.asm -o boot.img
$ qemu-system-<i386|x86_64> -drive format=raw,file=boot.img
```

Which should result in output like the following:
![screenshot](https://user-images.githubusercontent.com/12636891/65947844-15040600-e407-11e9-877a-76e7dd11bfa7.png)