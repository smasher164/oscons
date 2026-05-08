# OS Construction

This branch contains a BIOS-based bootloader for x86 that prints "Hello, World!" to the screen, written in Rust. [Cargo](https://doc.rust-lang.org/cargo/) is used to build the source against a custom [`i386-unknown-none-code16`](i386-none-16bit.json) target, which instructs LLVM to emit 16-bit x86 code suitable for real mode. `objcopy` is used to strip the ELF headers from the resulting binary. For convenience, these tools are configured in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. Build and run with:

```
$ make qemu
```
