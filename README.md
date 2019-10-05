# OS Construction

This branch contains a BIOS-based bootloader for x86 that prints enters protected mode. YASM is used to assemble the source, and is available at https://yasm.tortall.net. QEMU is used to emulate the hardware and BIOS necessary to run the bootloader, and is available at https://www.qemu.org/.

QEMU can be used to execute the flat binary file by treating it as a disk. Either qemu-system-i386 or qemu-system-x86_64 can be used to run the disk image. The following commands can be used to build and run the bootloader in QEMU:

```
$ yasm -f bin boot.asm -o boot.img
$ qemu-system-<i386|x86_64> -drive format=raw,file=boot.img
```

Successful boot into protected mode should result in output like:
![success](https://user-images.githubusercontent.com/12636891/66259375-52a5bd80-e77e-11e9-8ad9-91f7bc074738.png)

Whereas failure to boot into protected mode should result in output like:
![failure](https://user-images.githubusercontent.com/12636891/66259384-5e917f80-e77e-11e9-9bb0-e510804d2da4.png)