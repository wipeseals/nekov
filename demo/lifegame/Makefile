# Makefile for Conway's Game of Life for RISC-V

CC = riscv64-unknown-elf-gcc
OBJCOPY = riscv64-unknown-elf-objcopy
CFLAGS = -march=rv32ima -mabi=ilp32 -nostdlib -nostartfiles -ffreestanding -Os
LDFLAGS = -T linker.ld

# Target binary
TARGET = lifegame
SOURCES = lifegame.c

# Default target
all: $(TARGET).bin

# Object file
$(TARGET).o: $(SOURCES)
	$(CC) $(CFLAGS) -c -o $@ $<

# ELF file
$(TARGET).elf: $(TARGET).o linker.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<

# Binary file (raw)
$(TARGET).bin: $(TARGET).elf
	$(OBJCOPY) -O binary $< $@

# Clean
clean:
	rm -f *.o *.elf *.bin

.PHONY: all clean