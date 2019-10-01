# OS Construction

This branch contains a BIOS-based bootloader for x86 that prints "Hello, World!" to the screen. YASM is used to assemble the source, and is available at https://yasm.tortall.net. QEMU is used to emulate the hardware and BIOS necessary to run the bootloader, and is available at https://www.qemu.org/.

QEMU can be used to execute the flat binary file by treating it as a disk. Either qemu-system-i386 or qemu-system-x86_64 can be used to run the disk image. The following commands can be used to build and run the bootloader in QEMU:

```
$ yasm -f bin boot.s -o boot.img
$ qemu-system-<i386|x86_64> -hda boot.img
```

Which should result in output like the following:
![screenshot](https://user-images.githubusercontent.com/12636891/65946766-d705e280-e404-11e9-92d2-0abeb4f59641.png)