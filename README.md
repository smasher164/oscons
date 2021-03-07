# OS Construction

This branch contains a BIOS-based bootloader for x86 that enters protected mode. [YASM](https://yasm.tortall.net) is used to assemble the source, [clang](https://clang.llvm.org/)+[lld](https://lld.llvm.org/) are used to link the binary, and [objcopy](https://llvm.org/docs/CommandGuide/llvm-objcopy.html) is used to strip its ELF headers. For convenience, I have installed and configured these tools in a [Dockerfile](Dockerfile), whose image can be pulled as [smasher164/oscons:booting](https://hub.docker.com/r/smasher164/oscons/tags).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. It can execute a flat binary file by treating it as a disk. Use `qemu-system-i386` or `qemu-system-x86_64` to run the disk image. The following commands can be used to build and run the bootloader in QEMU:

```
$ yasm --oformat=elf boot.asm -o boot.o
$ cc -o boot.img boot.o --target=i386-none-elf -static -nostdlib -T script.ld
$ objcopy boot.img --strip-all --output-target=binary
$ qemu-system-<i386|x86_64> -drive format=raw,file=boot.img
```

Successful boot into protected mode should result in output like:
![success](https://user-images.githubusercontent.com/12636891/66259375-52a5bd80-e77e-11e9-8ad9-91f7bc074738.png)

Whereas failure to boot into protected mode should result in output like:
![failure](https://user-images.githubusercontent.com/12636891/66259384-5e917f80-e77e-11e9-9bb0-e510804d2da4.png)