/// ELF binary loading functionality
use crate::{memory::Memory, EmulatorError, Result};
use object::{Object, ObjectSection};
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

        // Load sections into memory
        for section in obj_file.sections() {
            // Only load sections that are allocated in memory
            let flags = section.flags();
            let is_alloc = match flags {
                object::SectionFlags::Elf { sh_flags } => sh_flags & (object::elf::SHF_ALLOC as u64) != 0,
                _ => false,
            };
            
            if !is_alloc {
                continue;
            }

            let address = section.address() as u32;
            let size = section.size();

            if size == 0 {
                continue;
            }

            // Get section data
            let section_data = section.data().map_err(|_| EmulatorError::InvalidElfFormat)?;

            // Load section into memory
            memory
                .load_data(address, section_data)
                .map_err(|_| EmulatorError::MemoryAccessError)?;

            println!(
                "Loaded section '{}' at 0x{:08x} (size: {} bytes)",
                section.name().unwrap_or("<unknown>"),
                address,
                size
            );
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