pub mod cpu;
pub mod elf_loader;
pub mod memory;
pub mod peripheral;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use std::path::Path;

#[derive(Debug)]
pub enum EmulatorError {
    FileNotFound,
    InvalidElfFormat,
    UnsupportedInstruction,
    MemoryAccessError,
    EcallTermination, // Normal termination via ECALL
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulatorError::FileNotFound => write!(f, "ELF file not found"),
            EmulatorError::InvalidElfFormat => write!(f, "Invalid ELF format"),
            EmulatorError::UnsupportedInstruction => write!(f, "Unsupported instruction"),
            EmulatorError::MemoryAccessError => write!(f, "Memory access error"),
            EmulatorError::EcallTermination => write!(f, "Normal termination via ECALL"),
        }
    }
}

impl std::error::Error for EmulatorError {}

pub type Result<T> = std::result::Result<T, EmulatorError>;

/// Main entry point for running the emulator
pub fn run_emulator(binary_path: &Path) -> Result<(cpu::Cpu, memory::Memory)> {
    run_emulator_with_limit(binary_path, Some(1000))
}

/// Run emulator with configurable instruction limit
pub fn run_emulator_with_limit(
    binary_path: &Path,
    instruction_limit: Option<usize>,
) -> Result<(cpu::Cpu, memory::Memory)> {
    run_emulator_with_limit_and_verbosity(binary_path, instruction_limit, 0)
}

/// Run emulator with configurable instruction limit and verbosity
pub fn run_emulator_with_limit_and_verbosity(
    binary_path: &Path,
    instruction_limit: Option<usize>,
    verbosity: u8,
) -> Result<(cpu::Cpu, memory::Memory)> {
    // Check if file exists
    if !binary_path.exists() {
        return Err(EmulatorError::FileNotFound);
    }

    // Initialize CPU and memory
    let mut cpu = cpu::Cpu::new();
    let mut memory = memory::Memory::new();

    // Load ELF binary into memory
    let entry_point = elf_loader::ElfLoader::load_elf(binary_path, &mut memory)?;

    // Set CPU program counter to entry point
    cpu.pc = entry_point;
    if verbosity >= 1 {
        println!("Entry point: 0x{entry_point:08x}");
    }

    // Run emulation with instruction limit for safety
    if verbosity >= 1 {
        println!("Starting emulation...");
    }
    let limit = instruction_limit.map(|l| l as u32);
    let executed_instructions = cpu.run_with_verbosity(&mut memory, limit, verbosity)?;
    if verbosity >= 1 {
        println!("Emulation completed. Executed {executed_instructions} instructions.");
    }

    // Print final CPU state if verbose
    if verbosity >= 2 {
        println!();
        println!("=== Final CPU State ===");
        println!("Final PC: 0x{:08x}", cpu.pc);
        println!("Registers:");
        for i in 0..8 {
            println!(
                "x{}: 0x{:08x}  x{}: 0x{:08x}  x{}: 0x{:08x}  x{}: 0x{:08x}",
                i,
                cpu.read_register(i),
                i + 8,
                cpu.read_register(i + 8),
                i + 16,
                cpu.read_register(i + 16),
                i + 24,
                cpu.read_register(i + 24)
            );
        }
    } else if verbosity == 0 {
        // Keep the old behavior for non-verbose mode
        println!("Entry point: 0x{entry_point:08x}");
        println!("Starting emulation...");
        println!("Emulation completed. Executed {executed_instructions} instructions.");
        println!("Final PC: 0x{:08x}", cpu.pc);
        println!("Registers:");
        for i in 0..8 {
            println!(
                "x{}: 0x{:08x}  x{}: 0x{:08x}  x{}: 0x{:08x}  x{}: 0x{:08x}",
                i,
                cpu.read_register(i),
                i + 8,
                cpu.read_register(i + 8),
                i + 16,
                cpu.read_register(i + 16),
                i + 24,
                cpu.read_register(i + 24)
            );
        }
    }

    Ok((cpu, memory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_run_emulator_file_not_found() {
        let non_existent_path = PathBuf::from("non_existent_file.elf");
        let result = run_emulator(&non_existent_path);
        assert!(matches!(result, Err(EmulatorError::FileNotFound)));
    }
}
