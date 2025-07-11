/// Memory management for the RISC-V emulator
use crate::EmulatorError;

/// Default memory size (4MB)
const DEFAULT_MEMORY_SIZE: usize = 4 * 1024 * 1024;

/// Memory implementation
#[derive(Debug, Clone)]
pub struct Memory {
    /// Memory data
    data: Vec<u8>,
    /// Base address
    base_address: u32,
}

impl Memory {
    /// Create a new memory instance with default size
    pub fn new() -> Self {
        Self::with_size(DEFAULT_MEMORY_SIZE)
    }

    /// Create a new memory instance with specified size
    pub fn with_size(size: usize) -> Self {
        Self {
            data: vec![0; size],
            base_address: 0x8000_0000, // Typical RISC-V RAM base address
        }
    }

    /// Read a byte from memory
    pub fn read_byte(&self, address: u32) -> Result<u8, EmulatorError> {
        let offset = self.address_to_offset(address)?;
        Ok(self.data[offset])
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, address: u32, value: u8) -> Result<(), EmulatorError> {
        let offset = self.address_to_offset(address)?;
        self.data[offset] = value;
        Ok(())
    }

    /// Read a 32-bit word from memory (little-endian)
    pub fn read_word(&self, address: u32) -> Result<u32, EmulatorError> {
        if address % 4 != 0 {
            return Err(EmulatorError::MemoryAccessError);
        }

        let offset = self.address_to_offset(address)?;
        if offset + 3 >= self.data.len() {
            return Err(EmulatorError::MemoryAccessError);
        }

        let value = u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ]);
        Ok(value)
    }

    /// Write a 32-bit word to memory (little-endian)
    pub fn write_word(&mut self, address: u32, value: u32) -> Result<(), EmulatorError> {
        if address % 4 != 0 {
            return Err(EmulatorError::MemoryAccessError);
        }

        let offset = self.address_to_offset(address)?;
        if offset + 3 >= self.data.len() {
            return Err(EmulatorError::MemoryAccessError);
        }

        let bytes = value.to_le_bytes();
        self.data[offset..offset + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Load data into memory at specified address
    pub fn load_data(&mut self, address: u32, data: &[u8]) -> Result<(), EmulatorError> {
        let offset = self.address_to_offset(address)?;
        if offset + data.len() > self.data.len() {
            return Err(EmulatorError::MemoryAccessError);
        }

        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// Get the base address of memory
    pub fn base_address(&self) -> u32 {
        self.base_address
    }

    /// Convert an address to an offset in the data array
    fn address_to_offset(&self, address: u32) -> Result<usize, EmulatorError> {
        if address < self.base_address {
            return Err(EmulatorError::MemoryAccessError);
        }

        let offset = (address - self.base_address) as usize;
        if offset >= self.data.len() {
            return Err(EmulatorError::MemoryAccessError);
        }

        Ok(offset)
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
        assert_eq!(memory.data.len(), DEFAULT_MEMORY_SIZE);
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
    fn test_memory_unaligned_word_access() {
        let mut memory = Memory::new();
        let base = memory.base_address();

        // Test unaligned access (should fail)
        let result = memory.write_word(base + 1, 0x12345678);
        assert!(matches!(result, Err(EmulatorError::MemoryAccessError)));

        let result = memory.read_word(base + 2);
        assert!(matches!(result, Err(EmulatorError::MemoryAccessError)));
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
    fn test_memory_out_of_bounds() {
        let mut memory = Memory::new();

        // Test access below base address
        let result = memory.read_byte(0x1000);
        assert!(matches!(result, Err(EmulatorError::MemoryAccessError)));

        // Test access beyond memory size
        let high_address = memory.base_address() + DEFAULT_MEMORY_SIZE as u32;
        let result = memory.read_byte(high_address);
        assert!(matches!(result, Err(EmulatorError::MemoryAccessError)));
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
