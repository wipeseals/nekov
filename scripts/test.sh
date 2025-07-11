#!/bin/bash

# Test script for nekov RISC-V emulator
# Runs all riscv-tests binaries through the emulator

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RISCV_TESTS_BINARIES_DIR="$PROJECT_DIR/riscv-tests-binaries"

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

# Create a simple Rust helper to run tests and manage output
cat > "$PROJECT_DIR/test_runner.rs" << 'EOF'
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: test_runner <emulator_path> <tests_dir>");
        exit(1);
    }
    
    let emulator_path = &args[1];
    let tests_dir = &args[2];
    
    let mut test_results = Vec::new();
    let mut total_tests = 0;
    let mut passed_tests = 0;
    
    println!("Running tests from: {}", tests_dir);
    println!();
    
    // Read all test files
    let entries = match fs::read_dir(tests_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read tests directory: {}", e);
            exit(1);
        }
    };
    
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        
        let filename = path.file_name().unwrap().to_string_lossy();
        
        // Skip files that are not test binaries (no extension typically)
        if filename.contains('.') {
            continue;
        }
        
        total_tests += 1;
        
        // Run the emulator on this test
        let output = Command::new(emulator_path)
            .arg(&path)
            .output();
            
        let (status, result_msg) = match output {
            Ok(output) => {
                if output.status.success() {
                    passed_tests += 1;
                    ("PASS", String::new())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    ("FAIL", format!("Exit code: {}, Error: {}", 
                                   output.status.code().unwrap_or(-1), 
                                   stderr.trim()))
                }
            }
            Err(e) => ("FAIL", format!("Failed to run: {}", e)),
        };
        
        test_results.push((filename.to_string(), status, result_msg));
    }
    
    // Print results
    println!("Test Results:");
    println!("=============");
    for (test_name, status, msg) in &test_results {
        let status_color = if *status == "PASS" { "\x1b[32m" } else { "\x1b[31m" };
        let reset_color = "\x1b[0m";
        print!("{}{:4}{} {}", status_color, status, reset_color, test_name);
        if !msg.is_empty() {
            print!(" - {}", msg);
        }
        println!();
    }
    
    println!();
    println!("Summary: {}/{} tests passed", passed_tests, total_tests);
    
    if passed_tests == total_tests {
        println!("🎉 All tests passed!");
        exit(0);
    } else {
        println!("❌ Some tests failed");
        exit(1);
    }
}
EOF

# Compile the test runner
rustc "$PROJECT_DIR/test_runner.rs" -o "$PROJECT_DIR/test_runner"

# Run the test runner
print_status "INFO" "Running RISC-V tests..."
echo

"$PROJECT_DIR/test_runner" "$PROJECT_DIR/target/release/nekov" "$TESTS_ISA_DIR"
test_exit_code=$?

# Clean up
rm -f "$PROJECT_DIR/test_runner.rs" "$PROJECT_DIR/test_runner"

exit $test_exit_code