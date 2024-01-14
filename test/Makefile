add-addi.bin: add-addi.s
	riscv64-unknown-elf-gcc -march=rv64g -Wl,-Ttext=0x0 -nostdlib -o bin/add-addi.elf add-addi.s
	riscv64-unknown-elf-objcopy -O binary bin/add-addi.elf bin/add-addi.bin

clean:
	rm -f bin/add-addi
	rm -f bin/add-addi.bin

dump:
	riscv64-unknown-linux-gnu-objdump -M no-aliases --disassembler-color=on --source --demangle --line-numbers --wide bin/add-addi.elf
