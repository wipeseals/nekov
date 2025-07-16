/// Peripheral abstraction for hardware interfacing
use crate::Result;

/// Trait for peripheral devices that can be attached to the CPU
pub trait Peripheral {
    /// Read from the peripheral at the given address offset
    fn read(&mut self, offset: u32) -> Result<u32>;

    /// Write to the peripheral at the given address offset
    fn write(&mut self, offset: u32, value: u32) -> Result<()>;

    /// Get the base address of this peripheral
    fn base_address(&self) -> u32;

    /// Get the size of the peripheral's address space
    fn size(&self) -> u32;

    /// Check if an address is within this peripheral's range
    fn contains_address(&self, address: u32) -> bool {
        let base = self.base_address();
        address >= base && address < base + self.size()
    }

    /// Flush any buffered output (for console peripherals)
    #[cfg(target_arch = "wasm32")]
    fn flush(&mut self) {
        // Default implementation does nothing
    }
}

/// Console peripheral for standard I/O
pub struct ConsolePeriph {
    base_addr: u32,
    #[cfg(target_arch = "wasm32")]
    buffer: String,
}

impl ConsolePeriph {
    pub fn new(base_addr: u32) -> Self {
        Self { 
            base_addr,
            #[cfg(target_arch = "wasm32")]
            buffer: String::new(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn flush(&mut self) {
        // Flush any remaining content in the buffer
        if !self.buffer.is_empty() {
            web_sys::console::log_1(&self.buffer.clone().into());
            self.buffer.clear();
        }
    }
}

impl Peripheral for ConsolePeriph {
    fn read(&mut self, _offset: u32) -> Result<u32> {
        // Console is write-only for now
        Ok(0)
    }

    fn write(&mut self, offset: u32, value: u32) -> Result<()> {
        match offset {
            0 => {
                // TX register - output character
                let ch = (value & 0xFF) as u8;
                #[cfg(target_arch = "wasm32")]
                {
                    let char = ch as char;
                    if char == '\n' {
                        // Log the complete line and clear the buffer
                        if !self.buffer.is_empty() {
                            web_sys::console::log_1(&self.buffer.clone().into());
                            self.buffer.clear();
                        } else {
                            // Log empty line for standalone newlines
                            web_sys::console::log_1(&"".into());
                        }
                    } else {
                        // Accumulate the character in the buffer
                        self.buffer.push(char);
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    print!("{}", ch as char);
                    use std::io::{self, Write};
                    io::stdout().flush().unwrap();
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn base_address(&self) -> u32 {
        self.base_addr
    }

    fn size(&self) -> u32 {
        0x1000 // 4KB address space
    }

    #[cfg(target_arch = "wasm32")]
    fn flush(&mut self) {
        // Flush any remaining content in the buffer
        if !self.buffer.is_empty() {
            web_sys::console::log_1(&self.buffer.clone().into());
            self.buffer.clear();
        }
    }
}

/// Peripheral manager to handle multiple peripherals
pub struct PeripheralManager {
    peripherals: Vec<Box<dyn Peripheral>>,
}

impl PeripheralManager {
    pub fn new() -> Self {
        Self {
            peripherals: Vec::new(),
        }
    }

    pub fn add_peripheral(&mut self, peripheral: Box<dyn Peripheral>) {
        self.peripherals.push(peripheral);
    }

    pub fn read(&mut self, address: u32) -> Result<u32> {
        for peripheral in &mut self.peripherals {
            if peripheral.contains_address(address) {
                let offset = address - peripheral.base_address();
                return peripheral.read(offset);
            }
        }
        // If no peripheral handles this address, return 0
        Ok(0)
    }

    pub fn write(&mut self, address: u32, value: u32) -> Result<()> {
        for peripheral in &mut self.peripherals {
            if peripheral.contains_address(address) {
                let offset = address - peripheral.base_address();
                return peripheral.write(offset, value);
            }
        }
        // If no peripheral handles this address, ignore the write
        Ok(())
    }

    pub fn is_peripheral_address(&self, address: u32) -> bool {
        self.peripherals.iter().any(|p| p.contains_address(address))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn flush_all(&mut self) {
        for peripheral in &mut self.peripherals {
            peripheral.flush();
        }
    }
}

impl Default for PeripheralManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_peripheral() {
        let mut console = ConsolePeriph::new(0x10000000);

        // Test base address and size
        assert_eq!(console.base_address(), 0x10000000);
        assert_eq!(console.size(), 0x1000);
        assert!(console.contains_address(0x10000000));
        assert!(console.contains_address(0x10000FFF));
        assert!(!console.contains_address(0x0FFFFFFF));
        assert!(!console.contains_address(0x10001000));

        // Test read (should return 0)
        assert_eq!(console.read(0).unwrap(), 0);

        // Test write (should succeed)
        assert!(console.write(0, b'H' as u32).is_ok());
        assert!(console.write(0, b'i' as u32).is_ok());
    }

    #[test]
    fn test_peripheral_manager() {
        let mut manager = PeripheralManager::new();

        // Add console peripheral
        let console = ConsolePeriph::new(0x10000000);
        manager.add_peripheral(Box::new(console));

        // Test address detection
        assert!(manager.is_peripheral_address(0x10000000));
        assert!(manager.is_peripheral_address(0x10000500));
        assert!(!manager.is_peripheral_address(0x20000000));

        // Test read/write
        assert_eq!(manager.read(0x10000000).unwrap(), 0);
        assert!(manager.write(0x10000000, b'A' as u32).is_ok());

        // Test non-peripheral address (should not fail)
        assert_eq!(manager.read(0x20000000).unwrap(), 0);
        assert!(manager.write(0x20000000, 0x12345678).is_ok());
    }
}
