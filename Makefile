STAGE16 = target/i386-none-16bit/release/libstage16.a
STAGE32 = target/i386-none-32bit/release/libstage32.a

boot.img: stage16/src/lib.rs stage32/src/lib.rs linker.ld i386-none-16bit.json i386-none-32bit.json
	cargo \
		-Z build-std=core \
		-Z build-std-features=compiler-builtins-mem \
		-Z json-target-spec \
		build --release -p stage16 --target i386-none-16bit.json
	cargo \
		-Z build-std=core \
		-Z build-std-features=compiler-builtins-mem \
		-Z json-target-spec \
		build --release -p stage32 --target i386-none-32bit.json
	ld.lld -T linker.ld --allow-multiple-definition $(STAGE16) $(STAGE32) -o boot.elf
	objcopy -O binary boot.elf boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	cargo clean
	-rm boot.img boot.elf
