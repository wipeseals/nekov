# nekov

A RISC-V emulator in Rust, probably written by a cat. 🐈

## Features

- **RV32I Base Integer Instruction Set**: Currently supports ADDI instruction with proper decoding
- **ELF Binary Loading**: Parses and loads ELF binaries into emulator memory  
- **Memory Management**: 4MB of emulated RAM with bounds checking and little-endian support
- **Register File**: 32 general-purpose registers (x0-x31) with x0 hardwired to zero
- **Instruction Execution**: Fetch-decode-execute cycle with instruction limits for safety

## Building

```bash
cargo build --release
```

## Running

```bash
# Run an ELF binary through the emulator
./target/release/nekov path/to/program.elf
```

## Testing

```bash
# Run unit tests
cargo test

# Run integration tests with sample programs
./scripts/test.sh
```

## Current Implementation Status

- ✅ **Basic Infrastructure**: CPU, memory, and register management
- ✅ **ELF Loading**: Parse ELF binaries and load them into memory
- ✅ **Instruction Decoding**: I-type instruction format parsing
- ✅ **ADDI Implementation**: Add immediate instruction with full test coverage
- ✅ **Test Suite**: Comprehensive unit tests and integration tests
- ✅ **CI/CD**: GitHub Actions for testing, linting, and formatting

### Supported Instructions

| Instruction | Format | Description | Status |
|-------------|--------|-------------|---------|
| ADDI | I-type | Add immediate to register | ✅ |

### Test Results

The emulator passes all tests including:
- Simple ADDI operations
- Instruction chaining  
- Negative immediate values
- Register file management
- Memory bounds checking

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
│ • Parse ELF     │    │ • 32 Registers  │    │ • 4MB RAM       │
│ • Load segments │    │ • Fetch/Decode  │    │ • Bounds Check  │
│ • Entry point   │    │ • Execute       │    │ • Little-endian │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality  
4. Ensure all tests pass: `cargo test && ./scripts/test.sh`
5. Submit a pull request

## License

MIT License (see LICENSE file)
