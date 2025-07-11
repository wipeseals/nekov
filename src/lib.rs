pub mod cpu;
pub mod memory;

use std::path::Path;

#[derive(Debug)]
pub enum EmulatorError {
    FileNotFound,
    InvalidElfFormat,
    UnsupportedInstruction,
    MemoryAccessError,
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmulatorError::FileNotFound => write!(f, "ELF file not found"),
            EmulatorError::InvalidElfFormat => write!(f, "Invalid ELF format"),
            EmulatorError::UnsupportedInstruction => write!(f, "Unsupported instruction"),
            EmulatorError::MemoryAccessError => write!(f, "Memory access error"),
        }
    }
}

impl std::error::Error for EmulatorError {}

pub type Result<T> = std::result::Result<T, EmulatorError>;

/// Main entry point for running the emulator
pub fn run_emulator(binary_path: &Path) -> Result<()> {
    // Check if file exists
    if !binary_path.exists() {
        return Err(EmulatorError::FileNotFound);
    }

    // Initialize CPU and memory
    let _cpu = cpu::Cpu::new();
    let _memory = memory::Memory::new();

    // TODO: Load ELF binary into memory
    println!("TODO: Load ELF binary: {}", binary_path.display());

    // TODO: Run emulation loop
    println!("TODO: Run emulation loop");

    Ok(())
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
