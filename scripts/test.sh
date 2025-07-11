#!/bin/bash

# Test script for nekov RISC-V emulator
# Creates simple test programs and runs them through the emulator

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
TEST_DIR="$SCRIPT_DIR/test_binaries"

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

# Create test directory
mkdir -p "$TEST_DIR"

# Function to create a simple test binary
create_test_binary() {
    local name=$1
    local hex_instructions=$2
    local description=$3
    
    print_status "INFO" "Creating test binary: $name ($description)"
    
    # Create a minimal ELF binary with our instructions
    # This is a simplified approach - in reality, we'd use the RISC-V toolchain
    python3 - << EOF
import struct

# Simple ELF header for RISC-V 32-bit
def create_simple_elf(instructions):
    # ELF header (52 bytes for 32-bit)
    elf_header = bytearray(52)
    
    # e_ident
    elf_header[0:4] = b'\x7fELF'  # magic
    elf_header[4] = 1  # EI_CLASS: 32-bit
    elf_header[5] = 1  # EI_DATA: little-endian
    elf_header[6] = 1  # EI_VERSION: current
    elf_header[7] = 0  # EI_OSABI: System V
    # EI_PAD: zeros (already set)
    
    # e_type: ET_EXEC (2)
    elf_header[16:18] = struct.pack('<H', 2)
    # e_machine: EM_RISCV (243)
    elf_header[18:20] = struct.pack('<H', 243)
    # e_version: EV_CURRENT (1)
    elf_header[20:24] = struct.pack('<I', 1)
    # e_entry: entry point
    entry_point = 0x80000000
    elf_header[24:28] = struct.pack('<I', entry_point)
    # e_phoff: program header offset
    elf_header[28:32] = struct.pack('<I', 52)
    # e_shoff: section header offset (0 for now)
    elf_header[32:36] = struct.pack('<I', 0)
    # e_flags: 0
    elf_header[36:40] = struct.pack('<I', 0)
    # e_ehsize: ELF header size
    elf_header[40:42] = struct.pack('<H', 52)
    # e_phentsize: program header entry size
    elf_header[42:44] = struct.pack('<H', 32)
    # e_phnum: number of program header entries
    elf_header[44:46] = struct.pack('<H', 1)
    # e_shentsize: section header entry size
    elf_header[46:48] = struct.pack('<H', 0)
    # e_shnum: number of section header entries
    elf_header[48:50] = struct.pack('<H', 0)
    # e_shstrndx: section header string table index
    elf_header[50:52] = struct.pack('<H', 0)
    
    # Program header (32 bytes for 32-bit)
    prog_header = bytearray(32)
    # p_type: PT_LOAD (1)
    prog_header[0:4] = struct.pack('<I', 1)
    # p_offset: offset in file
    prog_header[4:8] = struct.pack('<I', 84)  # 52 + 32
    # p_vaddr: virtual address
    prog_header[8:12] = struct.pack('<I', entry_point)
    # p_paddr: physical address
    prog_header[12:16] = struct.pack('<I', entry_point)
    # p_filesz: size in file
    prog_header[16:20] = struct.pack('<I', len(instructions))
    # p_memsz: size in memory
    prog_header[20:24] = struct.pack('<I', len(instructions))
    # p_flags: PF_R | PF_X (5)
    prog_header[24:28] = struct.pack('<I', 5)
    # p_align: alignment
    prog_header[28:32] = struct.pack('<I', 4)
    
    return bytes(elf_header) + bytes(prog_header) + instructions

# Parse hex instructions
instruction_bytes = bytes.fromhex('$hex_instructions')

# Create ELF binary
elf_data = create_simple_elf(instruction_bytes)

# Write to file
with open('$TEST_DIR/$name.elf', 'wb') as f:
    f.write(elf_data)

print(f"Created {len(elf_data)} byte ELF file")
EOF

    if [ $? -eq 0 ]; then
        print_status "PASS" "Created test binary: $name.elf"
        return 0
    else
        print_status "FAIL" "Failed to create test binary: $name"
        return 1
    fi
}

# Function to run a test
run_test() {
    local name=$1
    local expected_result=$2
    local description=$3
    
    print_status "INFO" "Running test: $name ($description)"
    
    if [ ! -f "$TEST_DIR/$name.elf" ]; then
        print_status "FAIL" "Test binary $name.elf not found"
        return 1
    fi
    
    # Run the emulator
    local output
    output=$("$PROJECT_DIR/target/release/nekov" "$TEST_DIR/$name.elf" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        print_status "PASS" "$name - emulator ran successfully"
        echo "Output:"
        echo "$output" | sed 's/^/  /'
        return 0
    else
        print_status "FAIL" "$name - emulator failed with exit code $exit_code"
        echo "Output:"
        echo "$output" | sed 's/^/  /'
        return 1
    fi
}

# Test 1: Simple ADDI instruction
# addi x1, x0, 42 (0x02a00093)
create_test_binary "addi_simple" "9300a002" "Simple ADDI x1, x0, 42"

# Test 2: Chain of ADDI instructions
# addi x1, x0, 10  (0x00a00093)
# addi x2, x1, 5   (0x00508113)  
# addi x3, x2, -3  (0xffd10193)
create_test_binary "addi_chain" "9300a000138150009301d1ff" "Chain of ADDI instructions"

# Test 3: ADDI with negative immediate
# addi x1, x0, -1 (0xfff00093)
create_test_binary "addi_negative" "9300f0ff" "ADDI with negative immediate"

# Run tests
echo
print_status "INFO" "Starting test execution..."
echo

failed_tests=0
total_tests=3

run_test "addi_simple" "0" "Simple ADDI instruction" || ((failed_tests++))
echo
run_test "addi_chain" "0" "Chain of ADDI instructions" || ((failed_tests++))
echo
run_test "addi_negative" "0" "ADDI with negative immediate" || ((failed_tests++))

echo
echo "===================================="
if [ $failed_tests -eq 0 ]; then
    print_status "PASS" "All $total_tests tests passed! 🎉"
    exit 0
else
    print_status "FAIL" "$failed_tests out of $total_tests tests failed"
    exit 1
fi