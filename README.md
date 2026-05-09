# OS Construction

This branch contains a BIOS-based bootloader for x86 that loads a second stage from disk and prints "Hello from stage 2!" to the screen, written in Rust. Stage 1 fits within the MBR (the first 510 bytes at 0x7C00), reads one sector from disk into 0x7E00, and far-jumps to stage 2. Stage 2 resets the segment registers and prints the message. [Cargo](https://doc.rust-lang.org/cargo/) is used to build the source against a custom [`i386-none-16bit`](i386-none-16bit.json) target, which instructs LLVM to emit 16-bit x86 code suitable for real mode. `objcopy` is used to strip the ELF headers from the resulting binary. For convenience, these tools are configured in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. Build and run with:

```
$ make qemu
```
