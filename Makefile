boot.img: src/main.rs Cargo.toml linker.ld i386-none-16bit.json
	RUSTFLAGS="-C link-arg=-Tlinker.ld" cargo \
		-Z build-std=core \
		-Z build-std-features=compiler-builtins-mem \
		-Z json-target-spec \
		build --release --target i386-none-16bit.json
	objcopy -O binary target/i386-none-16bit/release/boot boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	cargo clean
	-rm boot.img
