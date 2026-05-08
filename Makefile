boot.img: src/main.rs Cargo.toml linker.ld i386-none-16bit.json
	cargo build --release
	objcopy -O binary target/i386-none-16bit/release/boot boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	cargo clean
	-rm boot.img
