/// WASM bindings for the RISC-V emulator
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::{cpu::Cpu, memory::Memory, peripheral::{PeripheralManager, ConsolePeriph}};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmEmulator {
    cpu: Cpu,
    memory: Memory,
    peripherals: PeripheralManager,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmEmulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmEmulator {
        // Initialize console for panic output
        console_error_panic_hook::set_once();
        
        let cpu = Cpu::new();
        let memory = Memory::new();
        let mut peripherals = PeripheralManager::new();
        
        // Add console peripheral at address 0x10000000 (standard UART base)
        let console = ConsolePeriph::new(0x10000000);
        peripherals.add_peripheral(Box::new(console));
        
        WasmEmulator {
            cpu,
            memory,
            peripherals,
        }
    }
    
    #[wasm_bindgen]
    pub fn load_binary(&mut self, data: &[u8]) -> Result<u32, JsValue> {
        // For simplicity, we'll implement a basic binary loader
        // In a real implementation, this would parse ELF format
        
        // Load binary at address 0x80000000 (typical RISC-V program start)
        let load_address = 0x80000000;
        
        for (i, &byte) in data.iter().enumerate() {
            let addr = load_address + i as u32;
            self.memory.write_byte(addr, byte)
                .map_err(|e| JsValue::from_str(&format!("Memory error: {}", e)))?;
        }
        
        // Set PC to load address
        self.cpu.pc = load_address;
        
        Ok(load_address)
    }
    
    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<bool, JsValue> {
        match self.cpu.step_with_peripherals(&mut self.memory, &mut self.peripherals) {
            Ok(()) => Ok(true),
            Err(crate::EmulatorError::EcallTermination) => Ok(false), // Normal termination
            Err(e) => Err(JsValue::from_str(&format!("CPU error: {}", e))),
        }
    }
    
    #[wasm_bindgen]
    pub fn run(&mut self, max_instructions: Option<u32>) -> Result<u32, JsValue> {
        self.cpu.run_with_peripherals(&mut self.memory, &mut self.peripherals, max_instructions)
            .map_err(|e| match e {
                crate::EmulatorError::EcallTermination => JsValue::from_str("Program terminated normally"),
                _ => JsValue::from_str(&format!("CPU error: {}", e)),
            })
    }
    
    #[wasm_bindgen]
    pub fn get_pc(&self) -> u32 {
        self.cpu.pc
    }
    
    #[wasm_bindgen]
    pub fn get_register(&self, reg: usize) -> u32 {
        if reg < 32 {
            self.cpu.read_register(reg)
        } else {
            0
        }
    }
    
    #[wasm_bindgen]
    pub fn set_register(&mut self, reg: usize, value: u32) {
        if reg < 32 {
            self.cpu.write_register(reg, value);
        }
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
        self.memory = Memory::new();
        self.peripherals = PeripheralManager::new();
        
        // Re-add console peripheral
        let console = ConsolePeriph::new(0x10000000);
        self.peripherals.add_peripheral(Box::new(console));
    }
    
    #[wasm_bindgen]
    pub fn read_memory(&self, address: u32) -> u32 {
        self.memory.read_word(address).unwrap_or(0)
    }
    
    #[wasm_bindgen]
    pub fn write_memory(&mut self, address: u32, value: u32) -> Result<(), JsValue> {
        self.memory.write_word(address, value)
            .map_err(|e| JsValue::from_str(&format!("Memory error: {}", e)))
    }
}

// WASM utility functions
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}