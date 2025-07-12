#!/bin/bash

# Test script for nekov RISC-V emulator
# Runs all riscv-tests binaries through the emulator

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RISCV_TESTS_BINARIES_DIR="$PROJECT_DIR/riscv-tests-binaries"
# Determine verbose flag
VERBOSE_FLAG="$1"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🐈 Nekov RISC-V Emulator Test Suite"
echo "===================================="

# Function to print colored output
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

# Build the emulator
print_status "INFO" "Building nekov emulator..."
cd "$PROJECT_DIR"
cargo build --release

if [ $? -ne 0 ]; then
    print_status "FAIL" "Failed to build emulator"
    exit 1
fi

print_status "PASS" "Emulator built successfully"

# Check if riscv-tests binaries exist
if [ ! -d "$RISCV_TESTS_BINARIES_DIR" ]; then
    print_status "INFO" "RISC-V test binaries not found, checking if we can build them..."
    
    # Check if we have the RISC-V toolchain
    if ! command -v riscv64-unknown-elf-gcc &> /dev/null && ! command -v riscv32-unknown-elf-gcc &> /dev/null; then
        print_status "FAIL" "RISC-V toolchain not found. Please install gcc-riscv64-unknown-elf or set up riscv-tests-binaries directory"
        print_status "INFO" "In CI, this directory should be provided by the cache"
        exit 1
    fi
    
    # Build riscv-tests
    cd "$PROJECT_DIR/riscv-tests"
    if [ ! -f configure ]; then
        autoconf
    fi
    ./configure --prefix="$RISCV_TESTS_BINARIES_DIR"
    make
    make install
    
    print_status "PASS" "RISC-V tests built successfully"
fi

# Check for test binaries
TESTS_ISA_DIR="$RISCV_TESTS_BINARIES_DIR/share/riscv-tests/isa"
if [ ! -d "$TESTS_ISA_DIR" ]; then
    print_status "FAIL" "No test binaries found in $TESTS_ISA_DIR"
    exit 1
fi

# Run the test runner using cargo
print_status "INFO" "Running RISC-V tests..."
echo

cd "$PROJECT_DIR"
# Create logs directory for CI artifacts
mkdir -p logs

# Run tests and capture output for CI artifacts
cargo run --bin test_runner "$PROJECT_DIR/target/release/nekov" "$TESTS_ISA_DIR" $VERBOSE_FLAG > logs/riscv-tests-results.log
test_exit_code=$?

# Also create a JSON summary for easier parsing  
cargo run --bin test_runner "$PROJECT_DIR/target/release/nekov" "$TESTS_ISA_DIR" --json > logs/riscv-tests-results.json 2>/dev/null || true

# Show the results on console by displaying the log
cat logs/riscv-tests-results.log

print_status "INFO" "Test results saved to logs/ directory"

exit $test_exit_code