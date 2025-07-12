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
}

/// Console peripheral for standard I/O
pub struct ConsolePeriph {
    base_addr: u32,
}

impl ConsolePeriph {
    pub fn new(base_addr: u32) -> Self {
        Self { base_addr }
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
                    web_sys::console::log_1(&format!("{}", ch as char).into());
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
}

impl Default for PeripheralManager {
    fn default() -> Self {
        Self::new()
    }
}