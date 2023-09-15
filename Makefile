
 all:
	cargo build
	espflash flash  target/riscv32imc-unknown-none-elf/debug/blinky --monitor