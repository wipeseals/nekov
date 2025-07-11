/// ELF binary loading functionality
use crate::{memory::Memory, EmulatorError, Result};
use object::{Object, ObjectSegment};
use std::fs;

/// ELF loader for loading binaries into emulator memory
pub struct ElfLoader;

impl ElfLoader {
    /// Load an ELF binary into memory
    pub fn load_elf(file_path: &std::path::Path, memory: &mut Memory) -> Result<u32> {
        // Read the ELF file
        let data = fs::read(file_path).map_err(|_| EmulatorError::FileNotFound)?;

        // Parse the ELF file
        let obj_file = object::File::parse(&*data).map_err(|_| EmulatorError::InvalidElfFormat)?;

        let entry_point = obj_file.entry() as u32;

        // Load segments into memory (program headers)
        for segment in obj_file.segments() {
            let vaddr = segment.address() as u32;
            let file_range = segment.file_range();
            let file_size = file_range.1;

            if file_size == 0 {
                continue;
            }

            // Get segment data
            let segment_data = segment
                .data()
                .map_err(|_| EmulatorError::InvalidElfFormat)?;

            // Load segment into memory
            memory
                .load_data(vaddr, segment_data)
                .map_err(|_| EmulatorError::MemoryAccessError)?;

            println!("Loaded segment at 0x{vaddr:08x} (size: {file_size} bytes)");
        }

        Ok(entry_point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;
    use std::io::Write;

    #[test]
    fn test_load_elf_file_not_found() {
        let mut memory = Memory::new();
        let non_existent_path = std::path::Path::new("non_existent.elf");

        let result = ElfLoader::load_elf(non_existent_path, &mut memory);
        assert!(matches!(result, Err(EmulatorError::FileNotFound)));
    }

    #[test]
    fn test_load_elf_invalid_format() {
        let mut memory = Memory::new();

        // Create a temporary invalid ELF file
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        temp_file.write_all(b"not an elf file").unwrap();

        let result = ElfLoader::load_elf(temp_file.path(), &mut memory);
        assert!(matches!(result, Err(EmulatorError::InvalidElfFormat)));
    }
}
