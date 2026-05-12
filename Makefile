CARGO = cargo -Z build-std=core -Z build-std-features=compiler-builtins-mem -Z json-target-spec build --release

boot.img: src/lib.rs stage16/src/lib.rs stage32/src/lib.rs stage64/src/lib.rs linker.ld stage64/linker64.ld stage16/i386-none-16bit.json stage32/i386-none-32bit.json stage64/x86_64-none-64bit.json
	$(CARGO) -p stage16 --target stage16/i386-none-16bit.json
	$(CARGO) -p stage32 --target stage32/i386-none-32bit.json
	$(CARGO) -p stage64 --target stage64/x86_64-none-64bit.json
	ld.lld -T linker.ld --allow-multiple-definition \
		target/i386-none-16bit/release/libstage16.a \
		target/i386-none-32bit/release/libstage32.a \
		-o boot12.elf
	STAGE64_BASE=$$(nm boot12.elf | grep ' stage64_entry$$' | awk '{print "0x"$$1}'); \
	VGA_ADDR=$$(nm boot12.elf | grep ' VGA$$' | awk '{print "0x"$$1}'); \
	ld.lld -T stage64/linker64.ld \
		-defsym STAGE64_BASE=$$STAGE64_BASE \
		-defsym VGA=$$VGA_ADDR \
		target/x86_64-none-64bit/release/libstage64.a \
		-o stage64.elf
	objcopy -O binary boot12.elf boot12.bin
	objcopy -O binary stage64.elf stage64.bin
	cat boot12.bin stage64.bin > boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	cargo clean
	-rm boot.img boot12.elf boot12.bin stage64.elf stage64.bin
