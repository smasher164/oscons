# OS Construction

This branch contains a BIOS-based bootloader for x86 that queries the E820 memory map in real mode and displays it via VGA after entering protected mode, written in Rust. The build is structured as a Cargo workspace with three crates: a root crate holding shared types (E820, GDT), [`stage16`](stage16/) for real-mode code, and [`stage32`](stage32/) for protected-mode code. Each stage is compiled against a custom target ([`i386-none-16bit`](stage16/i386-none-16bit.json) and [`i386-none-32bit`](stage32/i386-none-32bit.json)), then linked together by `ld.lld` and stripped to a raw binary by `objcopy`. For convenience, these tools are configured in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

The boot sequence is:
1. Stage 1 (MBR): saves the drive number, loads subsequent sectors from disk, and jumps to stage 2.
2. Stage 2 (real mode): queries the E820 memory map via BIOS interrupt, enables the A20 line, loads the GDT, and enters protected mode.
3. Stage 3 (protected mode): displays the memory map and a success message via VGA text mode.

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. Build and run with:

```
$ make qemu
```
