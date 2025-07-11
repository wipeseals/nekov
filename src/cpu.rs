/// RISC-V CPU implementation
use crate::EmulatorError;

/// RISC-V register count (x0-x31)
const NUM_REGISTERS: usize = 32;

/// RISC-V CPU state
#[derive(Debug, Clone)]
pub struct Cpu {
    /// General-purpose registers (x0-x31)
    pub registers: [u32; NUM_REGISTERS],
    /// Program counter
    pub pc: u32,
}

impl Cpu {
    /// Create a new CPU instance
    pub fn new() -> Self {
        Self {
            registers: [0; NUM_REGISTERS],
            pc: 0,
        }
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.registers = [0; NUM_REGISTERS];
        self.pc = 0;
    }

    /// Read a register value
    pub fn read_register(&self, reg: usize) -> u32 {
        if reg == 0 {
            0 // x0 is always zero
        } else if reg < NUM_REGISTERS {
            self.registers[reg]
        } else {
            0 // Invalid register returns 0
        }
    }

    /// Write a register value
    pub fn write_register(&mut self, reg: usize, value: u32) {
        if reg != 0 && reg < NUM_REGISTERS {
            self.registers[reg] = value;
        }
        // x0 cannot be written, invalid registers are ignored
    }

    /// Execute an ADDI instruction (Add Immediate)
    /// Format: addi rd, rs1, imm
    pub fn execute_addi(&mut self, rd: usize, rs1: usize, imm: i32) -> Result<(), EmulatorError> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let rs1_value = self.read_register(rs1);
        let result = rs1_value.wrapping_add(imm as u32);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4); // Increment PC by 4 bytes

        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_new() {
        let cpu = Cpu::new();
        assert_eq!(cpu.pc, 0);
        for i in 0..NUM_REGISTERS {
            assert_eq!(cpu.read_register(i), 0);
        }
    }

    #[test]
    fn test_register_read_write() {
        let mut cpu = Cpu::new();

        // Test writing and reading normal registers
        cpu.write_register(1, 42);
        assert_eq!(cpu.read_register(1), 42);

        // Test that x0 always returns 0
        cpu.write_register(0, 42);
        assert_eq!(cpu.read_register(0), 0);

        // Test invalid register access
        assert_eq!(cpu.read_register(100), 0);
        cpu.write_register(100, 42); // Should not panic
    }

    #[test]
    fn test_addi_instruction() {
        let mut cpu = Cpu::new();

        // Test basic ADDI
        cpu.write_register(1, 10);
        cpu.execute_addi(2, 1, 5).unwrap();
        assert_eq!(cpu.read_register(2), 15);
        assert_eq!(cpu.pc, 4);

        // Test ADDI with negative immediate
        cpu.execute_addi(3, 2, -3).unwrap();
        assert_eq!(cpu.read_register(3), 12);
        assert_eq!(cpu.pc, 8);

        // Test ADDI to x0 (should not change x0)
        cpu.execute_addi(0, 1, 100).unwrap();
        assert_eq!(cpu.read_register(0), 0);
    }

    #[test]
    fn test_addi_overflow() {
        let mut cpu = Cpu::new();

        // Test overflow wrapping
        cpu.write_register(1, u32::MAX);
        cpu.execute_addi(2, 1, 1).unwrap();
        assert_eq!(cpu.read_register(2), 0); // Should wrap around
    }

    #[test]
    fn test_addi_invalid_registers() {
        let mut cpu = Cpu::new();

        // Test invalid destination register
        let result = cpu.execute_addi(100, 1, 5);
        assert!(matches!(result, Err(EmulatorError::UnsupportedInstruction)));

        // Test invalid source register
        let result = cpu.execute_addi(1, 100, 5);
        assert!(matches!(result, Err(EmulatorError::UnsupportedInstruction)));
    }
}
