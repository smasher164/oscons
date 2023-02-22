boot.img: boot.asm
	yasm --oformat=elf boot.asm -o boot.o
	i686-elf-cc -o boot.img boot.o -Ttext 0x7C00 -nostdlib -lgcc
	i686-elf-objcopy boot.img --strip-all --output-target=binary
	dd if=/dev/null of=boot.img bs=1 count=1 seek=16896

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	-rm boot.o boot.img
