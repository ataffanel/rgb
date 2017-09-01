

all: bootstrap.bin

bootstrap.bin: bootstrap.ihx
	objcopy -Iihex $< -Obinary $@

bootstrap.ihx: bootstrap.asm
	sdldgb -i bootstrap bootstrap.asm

bootstrap.asm: bootstrap.s
	sdasgb -ol bootstrap bootstrap.s

clean:
	rm -f bootstrap.ihx bootstrap.asm bootstrap.lst
