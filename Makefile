CARGO = cargo -Z build-std=core -Z build-std-features=compiler-builtins-mem -Z json-target-spec build --release

boot.img: src/lib.rs stage16/src/lib.rs stage32/src/lib.rs stage64/src/lib.rs kernel/src/lib.rs linker.ld stage64/linker64.ld kernel/linker.ld stage16/i386-none-16bit.json stage32/i386-none-32bit.json stage64/x86_64-none-64bit.json kernel/x86_64-none-kernel.json
	$(CARGO) -p stage16 --target stage16/i386-none-16bit.json
	$(CARGO) -p stage32 --target stage32/i386-none-32bit.json
	$(CARGO) -p stage64 --target stage64/x86_64-none-64bit.json
	$(CARGO) -p kernel --target kernel/x86_64-none-kernel.json
	# Link stage16 + stage32 into a single flat binary. --allow-multiple-definition
	# is needed because both crates pull in the shared oscons library.
	ld.lld -T linker.ld --allow-multiple-definition \
		target/i386-none-16bit/release/libstage16.a \
		target/i386-none-32bit/release/libstage32.a \
		-o boot12.elf
	# Link the kernel as a position-independent ELF at virtual address 0.
	# stage64 will parse and expand it at runtime.
	ld.lld -T kernel/linker.ld --no-dynamic-linker --gc-sections \
		target/x86_64-none-kernel/release/libkernel.a \
		-o kernel.elf
	# stage64 references symbols defined in stage32 (PML4, PDPT, VGA,
	# MEMORY_MAP) and must be linked at the address where stage64_entry lands
	# in boot12's memory map. Extract all these addresses from boot12.elf and
	# pass them as linker symbols so stage64 can reach them without relying on
	# a shared link step. KERNEL_SIZE lets the linker script compute kernel_end
	# = boot_end + KERNEL_SIZE, giving stage64 a clean address label for the
	# byte just past the on-disk kernel image.
	STAGE64_BASE=$$(nm boot12.elf | grep ' stage64_entry$$' | awk '{print "0x"$$1}'); \
	VGA_ADDR=$$(nm boot12.elf | grep ' VGA$$' | awk '{print "0x"$$1}'); \
	PML4_ADDR=$$(nm boot12.elf | grep ' PML4$$' | awk '{print "0x"$$1}'); \
	PDPT_ADDR=$$(nm boot12.elf | grep ' PDPT$$' | awk '{print "0x"$$1}'); \
	MMAP_ADDR=$$(nm boot12.elf | grep ' MEMORY_MAP$$' | awk '{print "0x"$$1}'); \
	KERNEL_SIZE=$$(wc -c < kernel.elf | tr -d ' \t\n'); \
	ld.lld -T stage64/linker64.ld --gc-sections \
		-defsym STAGE64_BASE=$$STAGE64_BASE \
		-defsym VGA=$$VGA_ADDR \
		-defsym PML4=$$PML4_ADDR \
		-defsym PDPT=$$PDPT_ADDR \
		-defsym MEMORY_MAP=$$MMAP_ADDR \
		-defsym KERNEL_SIZE=$$KERNEL_SIZE \
		target/x86_64-none-64bit/release/libstage64.a \
		-o stage64.elf
	# Strip ELF headers; concatenate into a raw disk image.
	# boot.img layout: [boot12.bin][stage64.bin][kernel.elf]
	objcopy -O binary boot12.elf boot12.bin
	objcopy -O binary stage64.elf stage64.bin
	cat boot12.bin stage64.bin kernel.elf > boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img -cpu qemu64,+rdrand

clean:
	cargo clean
	-rm boot.img boot12.elf boot12.bin stage64.elf stage64.bin kernel.elf
