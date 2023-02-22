# OS Construction

This branch contains a BIOS-based bootloader for x86 that loads another sector from disk, and enters protected mode. [YASM](https://yasm.tortall.net) is used to assemble the source, `gcc`/`ld` to link the binary, and `objcopy` to strip its ELF headers. For convenience, I have configured these tools in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. It can execute a flat binary file by treating it as a disk. Use `qemu-system-i386` or `qemu-system-x86_64` to run the disk image. Build and run the bootloader with 
```
$ make qemu
```

Successful boot into protected mode should result in output like:
![success](https://user-images.githubusercontent.com/12636891/220494696-94834444-8fa3-4fcd-8e12-ee7ebeca8de5.png)

Whereas failure to boot into protected mode should result in output like:
![failure](https://user-images.githubusercontent.com/12636891/220494844-72c492f3-6ed1-4bdb-90dd-cdb81202bb2f.png)
