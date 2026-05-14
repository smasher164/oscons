# OS Construction

This branch extends the long-mode bootloader to load and execute a standalone kernel. The build is structured as a Cargo workspace with five crates: a root crate holding shared types (E820, GDT, page table constants, VGA, `BootInfo`), [`stage16`](stage16/) for real-mode code, [`stage32`](stage32/) for protected-mode code, [`stage64`](stage64/) for 64-bit long-mode code, and [`kernel`](kernel/) for the kernel entry point. Each stage is compiled against a custom target ([`i386-none-16bit`](stage16/i386-none-16bit.json), [`i386-none-32bit`](stage32/i386-none-32bit.json), [`x86_64-none-64bit`](stage64/x86_64-none-64bit.json), [`x86_64-none-kernel`](kernel/x86_64-none-kernel.json)), then linked in two phases: stage16 and stage32 are linked first, their output is inspected with `nm` to extract the addresses of shared symbols (stage64 entry point, VGA cursor, page tables, memory map), and stage64 is linked separately at those addresses. The kernel is linked as a position-independent ELF at virtual address 0. The final disk image is the concatenation of the boot binary, the stage64 binary, and the kernel ELF. These tools are configured in a [flake.nix](flake.nix) file, which can be run using [Nix](https://nixos.org/).

The boot sequence is:
1. Stage 1 (MBR): saves the drive number, loads subsequent sectors from disk, and jumps to stage 2.
2. Stage 2 (real mode): queries the E820 memory map via BIOS interrupt, enables the A20 line, loads the GDT, and enters protected mode.
3. Stage 3 (protected mode): displays the memory map via VGA text mode, sets up a PML4→PDPT identity map covering the first 1 GB using 1 GB pages, enables PAE and long mode (EFER.LME), enables paging, and far-jumps to the 64-bit code segment.
4. Stage 4 (long mode): picks a random upper-half PML4 slot (index 256–511) for the kernel, parses the kernel ELF appended to the disk image, expands each `PT_LOAD` segment into memory immediately after the disk image, walks `.rela.dyn` and applies each `R_X86_64_RELATIVE` entry so that absolute pointers baked into the kernel's `.data` and `.rodata` resolve to its upper-half virtual address, and far-jumps to the kernel entry point with a pointer to `BootInfo` (containing the kernel virtual base, memory map pointer, and PML4 pointer).
5. Kernel: receives `BootInfo` and runs in the upper half of the virtual address space.

[QEMU](https://www.qemu.org/) is used to emulate the hardware and BIOS necessary to run the bootloader. Build and run with:

```
$ make qemu
```
