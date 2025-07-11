/// RISC-V CPU implementation
use crate::{memory::Memory, EmulatorError, Result};

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

    /// Execute a single instruction
    pub fn step(&mut self, memory: &mut Memory) -> Result<()> {
        // Fetch instruction from memory
        let instruction = memory.read_word(self.pc)?;

        // Decode and execute instruction
        self.decode_and_execute(instruction)?;

        Ok(())
    }

    /// Decode and execute an instruction
    fn decode_and_execute(&mut self, instruction: u32) -> Result<()> {
        // Extract opcode (bits 0-6)
        let opcode = instruction & 0x7F;

        match opcode {
            0x13 => {
                // I-type instruction (ADDI, SLTI, etc.)
                self.execute_i_type(instruction)
            }
            _ => {
                // Unsupported instruction
                Err(EmulatorError::UnsupportedInstruction)
            }
        }
    }

    /// Execute I-type instructions
    fn execute_i_type(&mut self, instruction: u32) -> Result<()> {
        // Extract fields
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let imm = (instruction as i32) >> 20; // Sign-extend immediate

        match funct3 {
            0x0 => {
                // ADDI instruction
                self.execute_addi(rd, rs1, imm)
            }
            _ => {
                // Unsupported funct3
                Err(EmulatorError::UnsupportedInstruction)
            }
        }
    }

    /// Execute an ADDI instruction (Add Immediate)
    /// Format: addi rd, rs1, imm
    pub fn execute_addi(&mut self, rd: usize, rs1: usize, imm: i32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let rs1_value = self.read_register(rs1);
        let result = rs1_value.wrapping_add(imm as u32);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4); // Increment PC by 4 bytes

        Ok(())
    }

    /// Run the CPU until it encounters an error or reaches a halt condition
    pub fn run(&mut self, memory: &mut Memory, max_instructions: Option<u32>) -> Result<u32> {
        let mut executed_instructions = 0;

        loop {
            // Check instruction limit
            if let Some(max) = max_instructions {
                if executed_instructions >= max {
                    break;
                }
            }

            // Execute one instruction
            match self.step(memory) {
                Ok(()) => {
                    executed_instructions += 1;
                }
                Err(EmulatorError::UnsupportedInstruction) => {
                    println!("Unsupported instruction at PC: 0x{:08x}", self.pc);
                    break;
                }
                Err(e) => {
                    println!("Error at PC: 0x{:08x}: {e}", self.pc);
                    return Err(e);
                }
            }
        }

        Ok(executed_instructions)
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
    use crate::memory::Memory;

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

    #[test]
    fn test_decode_addi_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();

        // Set up initial state
        cpu.pc = memory.base_address();
        cpu.write_register(1, 10);

        // Create ADDI instruction: addi x2, x1, 5
        // Instruction format: [imm(12) | rs1(5) | funct3(3) | rd(5) | opcode(7)]
        // imm=5, rs1=1, funct3=0, rd=2, opcode=0x13
        let instruction: u32 = (5 << 20) | (1 << 15) | (0 << 12) | (2 << 7) | 0x13;

        // Write instruction to memory
        memory.write_word(cpu.pc, instruction).unwrap();

        // Execute instruction
        cpu.step(&mut memory).unwrap();

        // Verify results
        assert_eq!(cpu.read_register(2), 15); // 10 + 5
        assert_eq!(cpu.pc, memory.base_address() + 4);
    }

    #[test]
    fn test_unsupported_instruction() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();

        cpu.pc = memory.base_address();

        // Create an unsupported instruction (opcode 0x7F)
        let instruction: u32 = 0x7F;
        memory.write_word(cpu.pc, instruction).unwrap();

        // Should return UnsupportedInstruction error
        let result = cpu.step(&mut memory);
        assert!(matches!(result, Err(EmulatorError::UnsupportedInstruction)));
    }

    #[test]
    fn test_cpu_run_with_limit() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();

        cpu.pc = memory.base_address();

        // Create multiple ADDI instructions
        let instruction: u32 = (1 << 20) | (1 << 15) | (0 << 12) | (1 << 7) | 0x13; // addi x1, x1, 1

        // Write instructions to memory
        for i in 0..10 {
            memory.write_word(cpu.pc + i * 4, instruction).unwrap();
        }

        // Set initial register value
        cpu.write_register(1, 0);

        // Run with instruction limit
        let executed = cpu.run(&mut memory, Some(5)).unwrap();
        assert_eq!(executed, 5);
        assert_eq!(cpu.read_register(1), 5); // Should have incremented 5 times
    }
}
