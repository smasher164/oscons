# OS Construction

This branch contains a BIOS-based bootloader for x86 that enters protected mode, written in Rust. [Cargo](https://doc.rust-lang.org/cargo/) is used to build the source against two custom targets: [`i386-none-16bit`](stage16/i386-none-16bit.json) for real-mode code and [`i386-none-32bit`](stage32/i386-none-32bit.json) for protected-mode code. `ld.lld` links the two together into a single boot image, and `objcopy` strips the ELF headers. For convenience, these tools are configured in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. Build and run with:

```
$ make qemu
```
