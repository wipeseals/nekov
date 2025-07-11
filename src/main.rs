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
        .get_matches();

    let binary_path = matches.get_one::<PathBuf>("binary").unwrap();

    println!("Nekov RISC-V Emulator");
    println!("Loading ELF binary: {}", binary_path.display());

    // TODO: Initialize emulator and load ELF binary
    match nekov::run_emulator(binary_path) {
        Ok(_) => println!("Emulation completed successfully"),
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
