.PHONY: qemu clean

TARGET = i386-none-16bit

boot.img: src/main.rs Cargo.toml linker.ld $(TARGET).json
	cargo build --release
	objcopy -O binary target/$(TARGET)/release/boot boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	cargo clean
	-rm boot.img
