/// Memory management for the RISC-V emulator
use crate::EmulatorError;
use std::collections::HashMap;

/// Memory implementation using dictionary-based storage
#[derive(Debug, Clone)]
pub struct Memory {
    /// Memory data - only stores written bytes
    data: HashMap<u32, u8>,
    /// Base address
    base_address: u32,
}

impl Memory {
    /// Create a new memory instance
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            base_address: 0x8000_0000, // Typical RISC-V RAM base address
        }
    }

    /// Create a new memory instance (kept for API compatibility)
    pub fn with_size(_size: usize) -> Self {
        Self::new()
    }

    /// Read a byte from memory
    pub fn read_byte(&self, address: u32) -> Result<u8, EmulatorError> {
        match self.data.get(&address) {
            Some(&value) => Ok(value),
            None => {
                eprintln!("Warning: Reading from uninitialized memory address 0x{address:08x}, returning 0xFF");
                Ok(0xFF)
            }
        }
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, address: u32, value: u8) -> Result<(), EmulatorError> {
        self.data.insert(address, value);
        Ok(())
    }

    /// Read a 16-bit halfword from memory (little-endian, supports misaligned access)
    pub fn read_halfword(&self, address: u32) -> Result<u16, EmulatorError> {
        let byte0 = self.read_byte(address)?;
        let byte1 = self.read_byte(address + 1)?;

        let value = u16::from_le_bytes([byte0, byte1]);
        Ok(value)
    }

    /// Read a 32-bit word from memory (little-endian, supports misaligned access)
    pub fn read_word(&self, address: u32) -> Result<u32, EmulatorError> {
        let byte0 = self.read_byte(address)?;
        let byte1 = self.read_byte(address + 1)?;
        let byte2 = self.read_byte(address + 2)?;
        let byte3 = self.read_byte(address + 3)?;

        let value = u32::from_le_bytes([byte0, byte1, byte2, byte3]);
        Ok(value)
    }

    /// Write a 16-bit halfword to memory (little-endian, supports misaligned access)
    pub fn write_halfword(&mut self, address: u32, value: u16) -> Result<(), EmulatorError> {
        let bytes = value.to_le_bytes();
        self.write_byte(address, bytes[0])?;
        self.write_byte(address + 1, bytes[1])?;
        Ok(())
    }

    /// Write a 32-bit word to memory (little-endian, supports misaligned access)
    pub fn write_word(&mut self, address: u32, value: u32) -> Result<(), EmulatorError> {
        let bytes = value.to_le_bytes();
        self.write_byte(address, bytes[0])?;
        self.write_byte(address + 1, bytes[1])?;
        self.write_byte(address + 2, bytes[2])?;
        self.write_byte(address + 3, bytes[3])?;
        Ok(())
    }

    /// Load data into memory at specified address
    pub fn load_data(&mut self, address: u32, data: &[u8]) -> Result<(), EmulatorError> {
        for (i, &byte) in data.iter().enumerate() {
            self.write_byte(address + i as u32, byte)?;
        }
        Ok(())
    }

    /// Get the base address of memory
    pub fn base_address(&self) -> u32 {
        self.base_address
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_new() {
        let memory = Memory::new();
        assert_eq!(memory.base_address(), 0x8000_0000);
        // Dictionary-based memory starts empty
        assert!(memory.data.is_empty());
    }

    #[test]
    fn test_memory_byte_access() {
        let mut memory = Memory::new();
        let base = memory.base_address();

        // Test writing and reading bytes
        memory.write_byte(base, 0x42).unwrap();
        assert_eq!(memory.read_byte(base).unwrap(), 0x42);

        memory.write_byte(base + 1, 0xFF).unwrap();
        assert_eq!(memory.read_byte(base + 1).unwrap(), 0xFF);
    }

    #[test]
    fn test_memory_word_access() {
        let mut memory = Memory::new();
        let base = memory.base_address();

        // Test writing and reading words (aligned)
        memory.write_word(base, 0x12345678).unwrap();
        assert_eq!(memory.read_word(base).unwrap(), 0x12345678);

        // Test multiple words
        memory.write_word(base + 4, 0xDEADBEEF).unwrap();
        assert_eq!(memory.read_word(base + 4).unwrap(), 0xDEADBEEF);
        assert_eq!(memory.read_word(base).unwrap(), 0x12345678); // First word unchanged
    }

    #[test]
    fn test_memory_misaligned_access() {
        let mut memory = Memory::new();
        let base = memory.base_address();

        // Test misaligned halfword access (should work)
        memory.write_halfword(base + 1, 0x1234).unwrap();
        assert_eq!(memory.read_halfword(base + 1).unwrap(), 0x1234);

        // Test misaligned word access (should work)
        memory.write_word(base + 5, 0x12345678).unwrap();
        assert_eq!(memory.read_word(base + 5).unwrap(), 0x12345678);

        // Verify individual bytes
        assert_eq!(memory.read_byte(base + 5).unwrap(), 0x78); // LSB
        assert_eq!(memory.read_byte(base + 6).unwrap(), 0x56);
        assert_eq!(memory.read_byte(base + 7).unwrap(), 0x34);
        assert_eq!(memory.read_byte(base + 8).unwrap(), 0x12); // MSB
    }

    #[test]
    fn test_memory_load_data() {
        let mut memory = Memory::new();
        let base = memory.base_address();

        let data = vec![0x01, 0x02, 0x03, 0x04];
        memory.load_data(base, &data).unwrap();

        assert_eq!(memory.read_byte(base).unwrap(), 0x01);
        assert_eq!(memory.read_byte(base + 1).unwrap(), 0x02);
        assert_eq!(memory.read_byte(base + 2).unwrap(), 0x03);
        assert_eq!(memory.read_byte(base + 3).unwrap(), 0x04);
    }

    #[test]
    fn test_memory_uninitialized_read() {
        let memory = Memory::new();
        let base = memory.base_address();

        // Reading uninitialized memory should return 0xFF with warning
        assert_eq!(memory.read_byte(base).unwrap(), 0xFF);
        assert_eq!(memory.read_byte(base + 100).unwrap(), 0xFF);
        assert_eq!(memory.read_byte(0x1000).unwrap(), 0xFF); // Any address should work now
    }

    #[test]
    fn test_memory_read_uninitialized_word() {
        let memory = Memory::new();
        let base = memory.base_address();

        // Reading uninitialized word should return 0xFFFFFFFF (all bytes 0xFF)
        assert_eq!(memory.read_word(base).unwrap(), 0xFFFFFFFF);
    }

    #[test]
    fn test_little_endian_encoding() {
        let mut memory = Memory::new();
        let base = memory.base_address();

        // Write a word and check individual bytes
        memory.write_word(base, 0x12345678).unwrap();
        assert_eq!(memory.read_byte(base).unwrap(), 0x78); // LSB
        assert_eq!(memory.read_byte(base + 1).unwrap(), 0x56);
        assert_eq!(memory.read_byte(base + 2).unwrap(), 0x34);
        assert_eq!(memory.read_byte(base + 3).unwrap(), 0x12); // MSB
    }
}
