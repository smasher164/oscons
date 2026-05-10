CARGO = cargo -Z build-std=core -Z build-std-features=compiler-builtins-mem -Z json-target-spec build --release

boot.img: src/lib.rs stage16/src/lib.rs stage32/src/lib.rs linker.ld stage16/i386-none-16bit.json stage32/i386-none-32bit.json
	$(CARGO) -p stage16 --target stage16/i386-none-16bit.json
	$(CARGO) -p stage32 --target stage32/i386-none-32bit.json
	ld.lld -T linker.ld --allow-multiple-definition \
		target/i386-none-16bit/release/libstage16.a \
		target/i386-none-32bit/release/libstage32.a \
		-o boot.elf
	objcopy -O binary boot.elf boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	cargo clean
	-rm boot.img boot.elf
