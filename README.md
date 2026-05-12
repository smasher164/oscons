# OS Construction

This branch contains a BIOS-based bootloader for x86 that queries the E820 memory map in real mode, displays it via VGA after entering protected mode, sets up identity-mapped page tables, and transitions to 64-bit long mode, written in Rust. The build is structured as a Cargo workspace with four crates: a root crate holding shared types (E820, GDT, page table constants, VGA), [`stage16`](stage16/) for real-mode code, [`stage32`](stage32/) for protected-mode code, and [`stage64`](stage64/) for 64-bit long-mode code. Each stage is compiled against a custom target ([`i386-none-16bit`](stage16/i386-none-16bit.json), [`i386-none-32bit`](stage32/i386-none-32bit.json), [`x86_64-none-64bit`](stage64/x86_64-none-64bit.json)), then linked in two phases: stage16 and stage32 are linked first, their output is inspected with `nm` to locate the stage64 entry point and the shared VGA cursor, and stage64 is linked separately at those addresses. The two raw binaries are concatenated into the final disk image. These tools are configured in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

The boot sequence is:
1. Stage 1 (MBR): saves the drive number, loads subsequent sectors from disk, and jumps to stage 2.
2. Stage 2 (real mode): queries the E820 memory map via BIOS interrupt, enables the A20 line, loads the GDT, and enters protected mode.
3. Stage 3 (protected mode): displays the memory map via VGA text mode, sets up a PML4→PDPT identity map covering the first 1 GB using 1 GB pages, enables PAE and long mode (EFER.LME), enables paging, and far-jumps to the 64-bit code segment.
4. Stage 4 (long mode): clears the vestigial data segment registers and prints a confirmation message via VGA.

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. Build and run with:

```
$ make qemu
```
