#!/bin/bash

# Build script for RISC-V tests
# This script builds only the physical tests (rv32ui-p-*) needed for the emulator

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RISCV_TESTS_DIR="$PROJECT_DIR/riscv-tests"
OUTPUT_DIR="$PROJECT_DIR/riscv-tests-binaries"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "PASS")
            echo -e "${GREEN}✓ PASS:${NC} $message"
            ;;
        "FAIL")
            echo -e "${RED}✗ FAIL:${NC} $message"
            ;;
        "INFO")
            echo -e "${YELLOW}ℹ INFO:${NC} $message"
            ;;
    esac
}

# Check if RISC-V toolchain is available
if ! command -v riscv64-unknown-elf-gcc &> /dev/null && ! command -v riscv32-unknown-elf-gcc &> /dev/null; then
    print_status "FAIL" "RISC-V toolchain not found. Please install gcc-riscv64-unknown-elf"
    exit 1
fi

# Set toolchain prefix
if command -v riscv64-unknown-elf-gcc &> /dev/null; then
    RISCV_PREFIX="riscv64-unknown-elf-"
    print_status "INFO" "Using riscv64-unknown-elf toolchain"
elif command -v riscv32-unknown-elf-gcc &> /dev/null; then
    RISCV_PREFIX="riscv32-unknown-elf-"
    print_status "INFO" "Using riscv32-unknown-elf toolchain"
fi

print_status "INFO" "Building RISC-V tests for RV32I..."

cd "$RISCV_TESTS_DIR"

# Configure if needed
if [ ! -f Makefile ]; then
    print_status "INFO" "Configuring riscv-tests..."
    if [ ! -f configure ]; then
        autoconf
    fi
    ./configure --prefix="$OUTPUT_DIR" --with-xlen=32
fi

# Build only the physical rv32ui tests we need
cd isa
print_status "INFO" "Building rv32ui physical tests..."

# Only build the specific tests we need (physical tests only)
# This avoids the virtual tests that require full C library support
TESTS_TO_BUILD="rv32ui-p-simple rv32ui-p-add rv32ui-p-addi rv32ui-p-and rv32ui-p-andi rv32ui-p-auipc rv32ui-p-beq rv32ui-p-bge rv32ui-p-bgeu rv32ui-p-blt rv32ui-p-bltu rv32ui-p-bne rv32ui-p-fence_i rv32ui-p-jal rv32ui-p-jalr rv32ui-p-lb rv32ui-p-lbu rv32ui-p-lh rv32ui-p-lhu rv32ui-p-lw rv32ui-p-ld_st rv32ui-p-lui rv32ui-p-ma_data rv32ui-p-or rv32ui-p-ori rv32ui-p-sb rv32ui-p-sh rv32ui-p-sw rv32ui-p-st_ld rv32ui-p-sll rv32ui-p-slli rv32ui-p-slt rv32ui-p-slti rv32ui-p-sltiu rv32ui-p-sltu rv32ui-p-sra rv32ui-p-srai rv32ui-p-srl rv32ui-p-srli rv32ui-p-sub rv32ui-p-xor rv32ui-p-xori"

# Build using the correct toolchain and only 32-bit tests
XLEN=32 RISCV_PREFIX="$RISCV_PREFIX" make $TESTS_TO_BUILD

if [ $? -ne 0 ]; then
    print_status "FAIL" "Failed to build RISC-V tests"
    exit 1
fi

# Create output directory and copy tests
mkdir -p "$OUTPUT_DIR/share/riscv-tests/isa"
cp rv32ui-p-* "$OUTPUT_DIR/share/riscv-tests/isa/" 2>/dev/null || true

# Count built tests
BUILT_TESTS=$(ls rv32ui-p-* 2>/dev/null | wc -l)
print_status "PASS" "Built $BUILT_TESTS RISC-V test binaries"

# List the tests for verification
print_status "INFO" "Available tests:"
for test in rv32ui-p-*; do
    if [ -f "$test" ]; then
        echo "  - $test"
    fi
done

print_status "PASS" "RISC-V tests build completed successfully"