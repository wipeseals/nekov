# Nekov RISC-V Emulator Web Demo

This is a web-based demonstration of the Nekov RISC-V emulator running Conway's Game of Life compiled for RISC-V architecture.

## Features

- **RISC-V RV32IMA Emulation**: Complete support for base integer, multiplication, and atomic instruction sets
- **WebAssembly Integration**: Emulator compiled to WASM for browser execution
- **Memory-Mapped Peripherals**: Console output via UART peripheral simulation
- **Interactive Controls**: Step-by-step execution, full program runs, and state inspection
- **Conway's Game of Life Demo**: Classic cellular automaton running on emulated RISC-V

## How to Use

1. **Load Demo**: Click "Load Life Game Demo" to load the Conway's Game of Life program
2. **Run Program**: Click "Run Program" to execute the complete demo
3. **Step Execution**: Use "Step" button for instruction-by-instruction execution
4. **Monitor State**: View CPU registers, program counter, and console output in real-time
5. **Load Custom Binaries**: Upload your own RISC-V binaries using the file input

## Technical Details

### Architecture
- **CPU**: 32-bit RISC-V with 32 general-purpose registers
- **Memory**: HashMap-based memory system with bounds checking
- **Peripherals**: Memory-mapped UART at address `0x10000000`
- **Program Loading**: Binaries loaded at address `0x80000000`

### Peripheral Interface
The emulator includes a simple UART peripheral for console output:
- **Base Address**: `0x10000000`
- **TX Register**: Offset `0x0` - Write characters to output
- **Behavior**: Writes to TX register appear in the web console

### Program Format
Programs should be compiled for RISC-V 32-bit with the following characteristics:
- **Architecture**: rv32ima (base + multiplication + atomics)
- **ABI**: ilp32
- **Entry Point**: `0x80000000`
- **System Calls**: Use ECALL with a7=93, a0=exit_code to terminate

## Example Program

```c
#include <stdint.h>

#define UART_BASE 0x10000000
#define UART_TX   (*(volatile uint32_t*)(UART_BASE + 0))

void uart_putc(char c) {
    UART_TX = (uint32_t)c;
}

int main() {
    const char* msg = "Hello from RISC-V!\n";
    for (int i = 0; msg[i]; i++) {
        uart_putc(msg[i]);
    }
    
    // Exit with ECALL
    __asm__ volatile("li a7, 93");  // sys_exit
    __asm__ volatile("li a0, 0");   // exit code 0
    __asm__ volatile("ecall");
    
    return 0;
}
```

## Building from Source

### Prerequisites
- Rust toolchain with `wasm32-unknown-unknown` target
- `wasm-pack` for building WebAssembly packages

### Build Steps
```bash
# Install WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Build the WASM package
wasm-pack build --target web --out-dir demo/web/pkg

# Serve the demo (requires a local web server)
# Example with Python:
cd demo/web
python3 -m http.server 8000
```

### Cross-compiling RISC-V Programs
To compile C programs for RISC-V:

```bash
# Install RISC-V toolchain
sudo apt-get install gcc-riscv64-unknown-elf

# Compile with specific flags
riscv32-unknown-elf-gcc -march=rv32ima -mabi=ilp32 \
    -nostdlib -nostartfiles -ffreestanding \
    -T linker.ld -o program.elf program.c

# Extract binary
riscv32-unknown-elf-objcopy -O binary program.elf program.bin
```

## Deployment

The demo is automatically deployed to GitHub Pages via GitHub Actions. The workflow:
1. Builds the WASM package
2. Uploads the web demo files
3. Deploys to GitHub Pages

## License

MIT License - see the main repository for details.