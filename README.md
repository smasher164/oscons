# OS Construction

This branch contains a BIOS-based bootloader for x86 that queries the E820 memory map and prints each region to the screen, written in Rust. Stage 1 fits within the MBR (the first 510 bytes at 0x7C00), reads seven sectors from disk into 0x7E00, and far-jumps to stage 2. Stage 2 resets the segment registers, calls BIOS INT 0x15/EAX=0xE820 in a loop to enumerate physical memory regions, and prints each entry's base address, length, type, and ACPI attributes. [Cargo](https://doc.rust-lang.org/cargo/) is used to build the source against a custom [`i386-none-16bit`](i386-none-16bit.json) target, which instructs LLVM to emit 16-bit x86 code suitable for real mode. `objcopy` is used to strip the ELF headers from the resulting binary. For convenience, these tools are configured in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. Build and run with:

```
$ make qemu
```
