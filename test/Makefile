rvlatortest.bin: baseinst.s
	riscv64-unknown-elf-gcc -march=rv64g -nostdlib -Wl,-Ttext=0x0 baseinst.s -o bin/rvlatortest.elf 
	riscv64-unknown-elf-objcopy -O binary bin/rvlatortest.elf bin/rvlatortest.bin

qemutest: baseinst.s
	riscv64-unknown-elf-gcc -march=rv64g -Wl,-Ttext=0x80000000 -nostdlib -o bin/qemutest.elf baseinst.s -g
	riscv64-unknown-elf-objcopy -O binary bin/qemutest.elf bin/qemutest.bin

clean:
	rm -f bin/rvlatortest.elf
	rm -f bin/qemutest.elf
	rm -f bin/rvlatortest.bin
	rm -f bin/qemutest.bin

debug:
	$(BUILDHUB_QEMU)/test/qemu-system-riscv64 -nographic -machine virt -bios none -kernel  bin/rvlatortest.elf -s -S

dump:
	riscv64-unknown-linux-gnu-objdump -M no-aliases --disassembler-color=on --source --demangle --line-numbers --wide bin/rvlatortest.elf
