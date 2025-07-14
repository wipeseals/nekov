# nekov

[![CI](https://github.com/wipeseals/nekov/workflows/CI/badge.svg)](https://github.com/wipeseals/nekov/actions)

A RISC-V emulator in Rust, probably written by a cat. 🐈

## Features

- **RV32IMA Instruction Set Support**: Complete implementation of RISC-V base integer (RV32I), multiplication (RV32M), and atomic (RV32A) instruction sets
- **40+ Instructions Implemented**: All major RISC-V instruction types including arithmetic, logical, memory, branch, jump, and atomic operations
- **ELF Binary Loading**: Parses and loads ELF binaries into emulator memory
- **Memory Management**: HashMap-based memory with bounds checking and little-endian support
- **Register File**: 32 general-purpose registers (x0-x31) with x0 hardwired to zero
- **Instruction Execution**: Fetch-decode-execute cycle with instruction limits for safety
- **RISC-V Tests Integration**: Support for running official RISC-V test suite
- **WebAssembly Support**: Full WASM compilation for browser execution
- **Peripheral System**: Memory-mapped peripheral support with console output
- **Web Demo**: Interactive Conway's Game of Life running in the browser

## Building

```bash
cargo build --release
```

## Web Demo

Try the emulator in your browser! The web demo features Conway's Game of Life running on the RISC-V emulator compiled to WebAssembly.

🌐 **[Live Demo](https://wipeseals.github.io/nekov)** (GitHub Pages)

### Running Locally

```bash
# Quick start - builds and serves the demo
./demo/serve.sh

# Manual build
wasm-pack build --target web --out-dir demo/web/pkg
cd demo/web && python3 -m http.server 8000
```

The web demo includes:
- Interactive RISC-V emulator controls
- Conway's Game of Life demonstration  
- Real-time CPU state monitoring
- Console output from memory-mapped UART
- Support for uploading custom RISC-V binaries

## Running

```bash
# Run an ELF binary through the emulator
./target/release/nekov path/to/program.elf
```

### Example Usage

```bash
./target/debug/nekov ./riscv-tests-binaries/share/riscv-tests/isa/rv32ui-p-addi -vvv --riscv-tests
Nekov RISC-V Emulator
Loading ELF binary: ./riscv-tests-binaries/share/riscv-tests/isa/rv32ui-p-addi
RISC-V tests mode enabled
Verbose output level: 3
Loaded segment at 0x80000000 (size: 1148 bytes)
Loaded segment at 0x80001000 (size: 72 bytes)
Entry point: 0x80000000
Starting emulation...
=== Starting CPU execution (verbose level 3) ===

Cycle 1: PC=0x80000000
  Instruction: 0x0500006f
  Before: x1=0x00000000 x2=0x00000000 x3=0x00000000 x10=0x00000000
  Fetched instruction: 0x0500006f
  Opcode: 0x6f
  JAL instruction
  After:  x1=0x00000000 x2=0x00000000 x3=0x00000000 x10=0x00000000

(snipped)

Cycle 277: PC=0x80000440
  Instruction: 0x00000073
  Before: x1=0x00000021 x2=0x00000000 x3=0x00000001 x10=0x00000000
  Fetched instruction: 0x00000073
  Opcode: 0x73
  System instruction
ECALL termination at PC: 0x80000440
=== CPU execution completed ===
Total instructions executed: 277
Emulation completed. Executed 277 instructions.

=== Final CPU State ===
Final PC: 0x80000440
Registers:
x0: 0x00000000  x8: 0x00000000  x16: 0x00000000  x24: 0x00000000
x1: 0x00000021  x9: 0x00000000  x17: 0x0000005d  x25: 0x00000000
x2: 0x00000000  x10: 0x00000000  x18: 0x00000000  x26: 0x00000000
x3: 0x00000001  x11: 0x00000018  x19: 0x00000000  x27: 0x00000000
x4: 0x00000002  x12: 0x00000000  x20: 0x00000000  x28: 0x00000000
x5: 0x00000002  x13: 0x7fffffff  x21: 0x00000000  x29: 0x00000000
x6: 0x00000016  x14: 0x00000016  x22: 0x00000000  x30: 0x00000000
x7: 0x00000000  x15: 0x00000000  x23: 0x00000000  x31: 0x00000000
=== RISC-V Test Result Analysis ===
Register state at termination:
  gp (x3)  = 0x00000001 (TESTNUM)
  a0 (x10) = 0x00000000 (exit code)
  a7 (x17) = 0x0000005d (syscall number)

Test result determination:
  ✓ System call number is 93 (exit syscall)
  ✓ TESTNUM=1 and exit code=0 → PASS
RISC-V test PASSED
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests with sample programs
./scripts/test.sh

# Run instruction verification test
cargo run --bin instruction_test
```

## Current Implementation Status

- ✅ **Basic Infrastructure**: CPU, memory, and register management
- ✅ **ELF Loading**: Parse ELF binaries and load them into memory
- ✅ **Instruction Decoding**: All RISC-V instruction formats (R, I, S, B, U, J)
- ✅ **RV32I Implementation**: Complete base integer instruction set (40+ instructions)
- ✅ **RV32M Implementation**: Multiplication and division extension (8 instructions)
- ✅ **RV32A Implementation**: Atomic instruction extension (11 instructions)
- ✅ **Test Suite**: Comprehensive unit tests for all instruction categories (27 tests)
- ✅ **CI/CD**: GitHub Actions for testing, linting, and formatting
- ✅ **WebAssembly**: Browser execution with full emulator functionality
- ✅ **Peripherals**: Memory-mapped device support with console output
- ✅ **Web Demo**: Conway's Game of Life running in the browser

### Supported Instructions

#### RV32I Base Integer Instructions

| Category              | Instructions                       | Status |
| --------------------- | ---------------------------------- | ------ |
| **I-type Arithmetic** | ADDI, SLTI, SLTIU, ANDI, ORI, XORI | ✅     |
| **I-type Shifts**     | SLLI, SRLI, SRAI                   | ✅     |
| **R-type Arithmetic** | ADD, SUB, SLT, SLTU                | ✅     |
| **R-type Logical**    | AND, OR, XOR                       | ✅     |
| **R-type Shifts**     | SLL, SRL, SRA                      | ✅     |
| **Load**              | LB, LH, LW, LBU, LHU               | ✅     |
| **Store**             | SB, SH, SW                         | ✅     |
| **Branch**            | BEQ, BNE, BLT, BGE, BLTU, BGEU     | ✅     |
| **Jump**              | JAL, JALR                          | ✅     |
| **Upper Immediate**   | LUI, AUIPC                         | ✅     |
| **System**            | ECALL, EBREAK                      | ✅     |

#### RV32M Multiplication Extension

| Category           | Instructions             | Status |
| ------------------ | ------------------------ | ------ |
| **Multiplication** | MUL, MULH, MULHSU, MULHU | ✅     |
| **Division**       | DIV, DIVU, REM, REMU     | ✅     |

#### RV32A Atomic Extension

| Category                | Instructions                             | Status |
| ----------------------- | ---------------------------------------- | ------ |
| **Load/Store Reserved** | LR.W, SC.W                               | ✅     |
| **Atomic Memory Ops**   | AMOSWAP.W, AMOADD.W, AMOXOR.W            | ✅     |
| **Atomic Logical**      | AMOAND.W, AMOOR.W                        | ✅     |
| **Atomic Min/Max**      | AMOMIN.W, AMOMAX.W, AMOMINU.W, AMOMAXU.W | ✅     |

**Total: 50+ instructions implemented covering RV32IMA**

### Peripheral System

The emulator includes a flexible peripheral system for hardware simulation:

| Peripheral | Base Address | Description |
|------------|--------------|-------------|
| **Console UART** | 0x10000000 | Character output to console/browser |

#### Memory Map
- **Program Memory**: 0x80000000+ (loaded binaries)
- **Console UART**: 0x10000000-0x10000FFF (4KB range)
- **General Memory**: Other addresses as needed

#### UART Interface
```c
#define UART_BASE 0x10000000
#define UART_TX   (*(volatile uint32_t*)(UART_BASE + 0))

void putchar(char c) {
    UART_TX = (uint32_t)c;  // Write character to console
}
```

### Test Results

The emulator passes all 27 unit tests including:

- **I-type instructions**: ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI
- **R-type instructions**: ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND
- **M-type instructions**: MUL, MULH, DIV, DIVU, REM, REMU with proper overflow handling
- **Load/Store operations**: All memory access patterns with proper alignment
- **Branch instructions**: Conditional branching with correct offset calculation
- **Jump instructions**: JAL and JALR with return address handling
- **Upper immediate**: LUI and AUIPC with proper immediate placement
- **Atomic operations**: Complete RV32A instruction set with memory synchronization
- **Edge cases**: Division by zero, arithmetic overflow, memory bounds checking

## Development

This project follows a TDD (Test-Driven Development) approach with:

- Unit tests for all components
- Integration tests using real instruction sequences
- CI pipeline ensuring code quality

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   ELF Loader    │───▶│      CPU        │───▶│     Memory      │
│                 │    │                 │    │                 │
│ • Parse ELF     │    │ • 32 Registers  │    │ • HashMap-based │
│ • Load segments │    │ • RV32IMA ISA   │    │ • Bounds Check  │
│ • Entry point   │    │ • Fetch/Decode  │    │ • Little-endian │
│                 │    │ • Execute 50+   │    │ • Atomic Ops    │
│                 │    │   Instructions  │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**Instruction Pipeline:**

1. **Fetch**: Read 32-bit instruction from memory at PC
2. **Decode**: Parse instruction format (R/I/S/B/U/J) and extract fields
3. **Execute**: Perform operation based on instruction type:
   - **Arithmetic**: ALU operations with proper overflow handling
   - **Memory**: Load/store with alignment checks and endianness
   - **Control**: Branches and jumps with target calculation
   - **Atomic**: Memory synchronization with acquire/release semantics

## Contributing

### Pull Request Requirements

Before submitting a PR, ensure the following requirements are met:

1. **Linting**: Code must pass all linting checks

   ```bash
   cargo clippy -- -D warnings
   ```

2. **Formatting**: Code must be properly formatted

   ```bash
   cargo fmt --check
   ```

3. **Build**: Code must build successfully

   ```bash
   cargo build
   ```

4. **CI**: All CI checks must pass (except for known failing tests that are documented)

These are mandatory requirements that will be checked before any PR review.

### Development Process

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test && ./scripts/test.sh`
5. Run lint, format, and build checks (see requirements above)
6. Submit a pull request

## License

MIT License (see LICENSE file)
