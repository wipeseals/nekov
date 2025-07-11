use std::env;
use std::fs;
use std::process::{exit, Command};

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

    println!("Running tests from: {tests_dir}");
    println!();

    // Read all test files
    let entries = match fs::read_dir(tests_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read tests directory: {e}");
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

        // Run the emulator on this test with riscv-tests mode
        let output = Command::new(emulator_path)
            .arg("--riscv-tests")
            .arg(&path)
            .output();

        let (status, result_msg) = match output {
            Ok(output) => {
                if output.status.success() {
                    passed_tests += 1;
                    ("PASS", String::new())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    (
                        "FAIL",
                        format!(
                            "Exit code: {}, Error: {}",
                            output.status.code().unwrap_or(-1),
                            stderr.trim()
                        ),
                    )
                }
            }
            Err(e) => ("FAIL", format!("Failed to run: {e}")),
        };

        test_results.push((filename.to_string(), status, result_msg));
    }

    // Print results
    println!("Test Results:");
    println!("=============");
    for (test_name, status, msg) in &test_results {
        let status_color = if *status == "PASS" {
            "\x1b[32m"
        } else {
            "\x1b[31m"
        };
        let reset_color = "\x1b[0m";
        print!("{status_color}{status:4}{reset_color} {test_name}");
        if !msg.is_empty() {
            print!(" - {msg}");
        }
        println!();
    }

    println!();
    println!("Summary: {passed_tests}/{total_tests} tests passed");

    if passed_tests == total_tests {
        println!("🎉 All tests passed!");
        exit(0);
    } else {
        println!("❌ Some tests failed");
        exit(1);
    }
}
