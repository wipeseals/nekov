use std::env;
use std::fs;
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        eprintln!("Usage: test_runner <emulator_path> <tests_dir> [--json]");
        exit(1);
    }

    let emulator_path = &args[1];
    let tests_dir = &args[2];
    let json_output = args.len() == 4 && args[3] == "--json";

    let mut test_results = Vec::new();
    let mut total_tests = 0;
    let mut passed_tests = 0;

    if !json_output {
        println!("🐈 Nekov RISC-V Test Runner");
        println!("===========================");
        println!("Running tests from: {tests_dir}");
        println!();
    }

    // Read all test files
    let entries = match fs::read_dir(tests_dir) {
        Ok(entries) => entries,
        Err(e) => {
            if json_output {
                println!("{{\"error\": \"Failed to read tests directory: {e}\"}}");
            } else {
                eprintln!("Failed to read tests directory: {e}");
            }
            exit(1);
        }
    };

    let mut entries_vec: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries_vec.sort_by_key(|a| a.file_name());

    for entry in entries_vec {
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

    if json_output {
        // Output JSON format for machine processing
        println!("{{");
        println!("  \"total_tests\": {total_tests},");
        println!("  \"passed_tests\": {passed_tests},");
        println!("  \"failed_tests\": {},", total_tests - passed_tests);
        println!(
            "  \"pass_rate\": {:.2},",
            if total_tests > 0 {
                passed_tests as f64 / total_tests as f64 * 100.0
            } else {
                0.0
            }
        );
        println!("  \"results\": [");
        for (i, (test_name, status, msg)) in test_results.iter().enumerate() {
            let comma = if i < test_results.len() - 1 { "," } else { "" };
            println!("    {{");
            println!("      \"test\": \"{test_name}\",");
            println!("      \"status\": \"{status}\",");
            println!("      \"message\": \"{}\"", msg.replace('"', "\\\""));
            println!("    }}{comma}");
        }
        println!("  ]");
        println!("}}");
    } else {
        // Print human-readable results
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
        println!(
            "Summary: {passed_tests}/{total_tests} tests passed ({:.1}% pass rate)",
            if total_tests > 0 {
                passed_tests as f64 / total_tests as f64 * 100.0
            } else {
                0.0
            }
        );

        if passed_tests == total_tests {
            println!("🎉 All tests passed!");
        } else {
            println!(
                "❌ {}/{} tests failed",
                total_tests - passed_tests,
                total_tests
            );

            // List failed tests for quick reference
            println!("\nFailed tests:");
            for (test_name, status, _) in &test_results {
                if *status == "FAIL" {
                    println!("  - {test_name}");
                }
            }
        }
    }

    if passed_tests == total_tests {
        exit(0);
    } else {
        exit(1);
    }
}
