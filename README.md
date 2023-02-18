# OS Construction

This branch contains a BIOS-based bootloader for x86 that prints "Hello, World!" to the screen. [YASM](https://yasm.tortall.net) is used to assemble the source, `gcc`/`ld` to link the binary, and `objcopy` to strip its ELF headers. For convenience, I have configured these tools in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. It can execute a flat binary file by treating it as a disk. Use `qemu-system-i386` or `qemu-system-x86_64` to run the disk image. Build and run the bootloader with 
```
$ make qemu
```

Which should result in output like the following:
![screenshot](https://user-images.githubusercontent.com/12636891/66261025-960b2680-e794-11e9-8982-1b473261ed10.png)
