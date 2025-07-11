use clap::{Arg, Command};
use std::path::PathBuf;

fn main() {
    let matches = Command::new("nekov")
        .version("0.1.0")
        .author("wipeseals")
        .about("A RISC-V emulator in Rust, probably written by a cat. 🐈")
        .arg(
            Arg::new("binary")
                .help("ELF binary file to emulate")
                .required(true)
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("limit")
                .long("limit")
                .short('l')
                .help("Maximum number of instructions to execute")
                .value_name("NUM")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("riscv-tests")
                .long("riscv-tests")
                .help("Enable riscv-tests pass/fail detection")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let binary_path = matches.get_one::<PathBuf>("binary").unwrap();
    let instruction_limit = matches.get_one::<usize>("limit").copied();
    let riscv_tests_mode = matches.get_flag("riscv-tests");

    println!("Nekov RISC-V Emulator");
    println!("Loading ELF binary: {}", binary_path.display());

    if let Some(limit) = instruction_limit {
        println!("Instruction limit: {limit}");
    }

    if riscv_tests_mode {
        println!("RISC-V tests mode enabled");
    }

    match nekov::run_emulator_with_limit(binary_path, instruction_limit) {
        Ok((cpu, _memory)) => {
            if riscv_tests_mode {
                // Check for riscv-tests pass/fail patterns
                let test_result = check_riscv_test_result(&cpu);
                match test_result {
                    TestResult::Pass => {
                        println!("RISC-V test PASSED");
                        std::process::exit(0);
                    }
                    TestResult::Fail(code) => {
                        println!("RISC-V test FAILED (code: 0x{code:x})");
                        std::process::exit(1);
                    }
                    TestResult::Unknown => {
                        println!("RISC-V test result: UNKNOWN");
                        std::process::exit(2);
                    }
                }
            } else {
                println!("Emulation completed successfully");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug, PartialEq)]
enum TestResult {
    Pass,
    Fail(u32),
    Unknown,
}

/// Check RISC-V test result based on register state
/// Based on RVTEST_PASS/RVTEST_FAIL macros:
/// - PASS: TESTNUM=1 (gp=1), a7=93, a0=0, ecall
/// - FAIL: TESTNUM!=1 (gp!=1), a7=93, a0=TESTNUM, ecall
fn check_riscv_test_result(cpu: &nekov::cpu::Cpu) -> TestResult {
    // Check if we ended with an ecall (system call)
    // Register assignments for RISC-V:
    // gp = x3 (TESTNUM)
    // a0 = x10 (first argument)
    // a7 = x17 (system call number)

    let testnum = cpu.read_register(3); // gp register
    let a0 = cpu.read_register(10); // a0 register
    let a7 = cpu.read_register(17); // a7 register

    // Check if this looks like a test termination (a7=93 is exit syscall)
    if a7 == 93 {
        if testnum == 1 && a0 == 0 {
            TestResult::Pass
        } else if testnum != 1 {
            TestResult::Fail(a0)
        } else {
            TestResult::Unknown
        }
    } else {
        TestResult::Unknown
    }
}
