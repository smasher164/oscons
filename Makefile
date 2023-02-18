boot.img: boot.asm
	yasm --oformat=elf boot.asm -o boot.o
	i686-elf-cc -o boot.img boot.o -Ttext 0x7C00 -nostdlib -lgcc
	i686-elf-objcopy boot.img --strip-all --pad-to=0x7DFE --output-target=binary
	echo -en "\x55\xAA" >> boot.img

qemu: boot.img
	qemu-system-x86_64 -drive format=raw,file=boot.img

clean:
	-rm boot.o boot.img
