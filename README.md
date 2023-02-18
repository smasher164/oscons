# OS Construction

This branch contains a BIOS-based bootloader for x86 that enters protected mode. [YASM](https://yasm.tortall.net) is used to assemble the source, `gcc`/`ld` to link the binary, and `objcopy` to strip its ELF headers. For convenience, I have configured these tools in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. It can execute a flat binary file by treating it as a disk. Use `qemu-system-i386` or `qemu-system-x86_64` to run the disk image. Build and run the bootloader with 
```
$ make qemu
```

Successful boot into protected mode should result in output like:
![success](https://user-images.githubusercontent.com/12636891/66259375-52a5bd80-e77e-11e9-8ad9-91f7bc074738.png)

Whereas failure to boot into protected mode should result in output like:
![failure](https://user-images.githubusercontent.com/12636891/66259384-5e917f80-e77e-11e9-9bb0-e510804d2da4.png)
