/// RISC-V CPU implementation
use crate::{memory::Memory, EmulatorError, Result};

/// Macro for verbose logging at different levels
macro_rules! verbose_log {
    ($verbosity:expr, $level:expr, $($arg:tt)*) => {
        if $verbosity >= $level {
            println!($($arg)*);
        }
    };
}

/// Macro for debug-level verbose logging (level 3)
macro_rules! debug_log {
    ($verbosity:expr, $($arg:tt)*) => {
        verbose_log!($verbosity, 3, $($arg)*);
    };
}

/// Macro for info-level verbose logging (level 2)
macro_rules! info_log {
    ($verbosity:expr, $($arg:tt)*) => {
        verbose_log!($verbosity, 2, $($arg)*);
    };
}

/// Macro for basic verbose logging (level 1)
macro_rules! basic_log {
    ($verbosity:expr, $($arg:tt)*) => {
        verbose_log!($verbosity, 1, $($arg)*);
    };
}

/// RISC-V register count (x0-x31)
const NUM_REGISTERS: usize = 32;

/// RISC-V CPU state
#[derive(Debug, Clone)]
pub struct Cpu {
    /// General-purpose registers (x0-x31)
    pub registers: [u32; NUM_REGISTERS],
    /// Program counter
    pub pc: u32,
    /// Control and Status Registers (CSRs)
    /// For simplicity, we'll store only the most common ones
    pub csrs: std::collections::HashMap<u16, u32>,
}

impl Cpu {
    /// Create a new CPU instance
    pub fn new() -> Self {
        let mut csrs = std::collections::HashMap::new();
        // Initialize commonly used CSRs
        csrs.insert(0xF14, 0); // mhartid - hardware thread ID
        csrs.insert(0x300, 0); // mstatus - machine status
        csrs.insert(0x341, 0); // mepc - machine exception program counter
        csrs.insert(0x342, 0); // mcause - machine trap cause
        csrs.insert(0x343, 0); // mtval - machine trap value
        csrs.insert(0x344, 0); // mip - machine interrupt pending
        csrs.insert(0x304, 0); // mie - machine interrupt enable
        csrs.insert(0x305, 0); // mtvec - machine trap-handler base address
        csrs.insert(0x340, 0); // mscratch - machine scratch register
        csrs.insert(0xF11, 0); // mvendorid - vendor ID
        csrs.insert(0xF12, 0); // marchid - architecture ID
        csrs.insert(0xF13, 0); // mimpid - implementation ID
        csrs.insert(0xC00, 0); // cycle - cycle counter
        csrs.insert(0xC01, 0); // time - time counter
        csrs.insert(0xC02, 0); // instret - instructions retired counter

        Self {
            registers: [0; NUM_REGISTERS],
            pc: 0,
            csrs,
        }
    }

    /// Reset the CPU to initial state
    pub fn reset(&mut self) {
        self.registers = [0; NUM_REGISTERS];
        self.pc = 0;
        // Reset CSRs to default values
        self.csrs.clear();
        self.csrs.insert(0xF14, 0); // mhartid
        self.csrs.insert(0x300, 0); // mstatus
        self.csrs.insert(0x341, 0); // mepc
        self.csrs.insert(0x342, 0); // mcause
        self.csrs.insert(0x343, 0); // mtval
        self.csrs.insert(0x344, 0); // mip
        self.csrs.insert(0x304, 0); // mie
        self.csrs.insert(0x305, 0); // mtvec
        self.csrs.insert(0x340, 0); // mscratch
        self.csrs.insert(0xF11, 0); // mvendorid
        self.csrs.insert(0xF12, 0); // marchid
        self.csrs.insert(0xF13, 0); // mimpid
        self.csrs.insert(0xC00, 0); // cycle
        self.csrs.insert(0xC01, 0); // time
        self.csrs.insert(0xC02, 0); // instret
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

    /// Read a CSR value
    pub fn read_csr(&self, csr: u16) -> u32 {
        self.csrs.get(&csr).copied().unwrap_or(0)
    }

    /// Write a CSR value
    pub fn write_csr(&mut self, csr: u16, value: u32) {
        self.csrs.insert(csr, value);
    }

    /// Execute a single instruction
    pub fn step(&mut self, memory: &mut Memory) -> Result<()> {
        self.step_with_verbosity(memory, 0)
    }

    /// Execute a single instruction with peripheral support
    pub fn step_with_peripherals(
        &mut self,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
    ) -> Result<()> {
        self.step_with_peripherals_and_verbosity(memory, peripherals, 0)
    }

    /// Execute a single instruction with verbose output
    pub fn step_with_verbosity(&mut self, memory: &mut Memory, verbosity: u8) -> Result<()> {
        // Fetch instruction from memory
        let instruction = memory.read_word(self.pc)?;

        debug_log!(verbosity, "  Fetched instruction: 0x{instruction:08x}");

        // Decode and execute instruction
        self.decode_and_execute_with_verbosity(instruction, memory, verbosity)?;

        Ok(())
    }

    /// Execute a single instruction with peripheral and verbose support
    pub fn step_with_peripherals_and_verbosity(
        &mut self,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
        verbosity: u8,
    ) -> Result<()> {
        // Fetch instruction from memory
        let instruction = memory.read_word(self.pc)?;

        debug_log!(verbosity, "  Fetched instruction: 0x{instruction:08x}");

        // Decode and execute instruction
        self.decode_and_execute_with_peripherals_and_verbosity(
            instruction,
            memory,
            peripherals,
            verbosity,
        )?;

        Ok(())
    }

    /// Decode and execute an instruction with verbose output
    fn decode_and_execute_with_verbosity(
        &mut self,
        instruction: u32,
        memory: &mut Memory,
        verbosity: u8,
    ) -> Result<()> {
        // Extract opcode (bits 0-6)
        let opcode = instruction & 0x7F;

        debug_log!(verbosity, "  Opcode: 0x{opcode:02x}");

        match opcode {
            0x13 => {
                // I-type instruction (ADDI, SLTI, XORI, etc.)
                debug_log!(verbosity, "  I-type instruction");
                self.execute_i_type(instruction)
            }
            0x33 => {
                // R-type instruction (ADD, SUB, XOR, etc.)
                debug_log!(verbosity, "  R-type instruction");
                self.execute_r_type(instruction)
            }
            0x03 => {
                // Load instructions (LB, LH, LW, LBU, LHU)
                debug_log!(verbosity, "  Load instruction");
                self.execute_load(instruction, memory)
            }
            0x23 => {
                // Store instructions (SB, SH, SW)
                debug_log!(verbosity, "  Store instruction");
                self.execute_store(instruction, memory)
            }
            0x63 => {
                // Branch instructions (BEQ, BNE, BLT, BGE, BLTU, BGEU)
                debug_log!(verbosity, "  Branch instruction");
                self.execute_branch(instruction)
            }
            0x37 => {
                // LUI instruction
                debug_log!(verbosity, "  LUI instruction");
                self.execute_lui(instruction)
            }
            0x17 => {
                // AUIPC instruction
                debug_log!(verbosity, "  AUIPC instruction");
                self.execute_auipc(instruction)
            }
            0x6F => {
                // JAL instruction
                debug_log!(verbosity, "  JAL instruction");
                self.execute_jal(instruction)
            }
            0x67 => {
                // JALR instruction
                debug_log!(verbosity, "  JALR instruction");
                self.execute_jalr(instruction)
            }
            0x73 => {
                // System instructions (ECALL, EBREAK)
                debug_log!(verbosity, "  System instruction");
                self.execute_system(instruction)
            }
            0x2F => {
                // RV32A atomic instructions
                debug_log!(verbosity, "  Atomic instruction");
                self.execute_atomic(instruction, memory)
            }
            0x0F => {
                // FENCE instruction family (memory ordering)
                debug_log!(verbosity, "  FENCE instruction");
                let funct3 = (instruction >> 12) & 0x7;
                match funct3 {
                    0x0 => {
                        // FENCE - memory fence
                        // For our simple emulator, we'll treat it as a no-op
                        self.pc = self.pc.wrapping_add(4);
                        Ok(())
                    }
                    0x1 => {
                        // FENCE.I - instruction fence
                        // For our simple emulator, we'll treat it as a no-op
                        self.pc = self.pc.wrapping_add(4);
                        Ok(())
                    }
                    _ => Err(EmulatorError::UnsupportedInstruction),
                }
            }
            _ => {
                // Unsupported instruction
                Err(EmulatorError::UnsupportedInstruction)
            }
        }
    }

    /// Decode and execute an instruction with peripheral and verbose support
    fn decode_and_execute_with_peripherals_and_verbosity(
        &mut self,
        instruction: u32,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
        verbosity: u8,
    ) -> Result<()> {
        // Extract opcode (bits 0-6)
        let opcode = instruction & 0x7F;

        debug_log!(verbosity, "  Opcode: 0x{opcode:02x}");

        match opcode {
            0x13 => {
                // I-type instruction (ADDI, SLTI, XORI, etc.)
                debug_log!(verbosity, "  I-type instruction");
                self.execute_i_type(instruction)
            }
            0x33 => {
                // R-type instruction (ADD, SUB, XOR, etc.)
                debug_log!(verbosity, "  R-type instruction");
                self.execute_r_type(instruction)
            }
            0x03 => {
                // Load instructions (LB, LH, LW, LBU, LHU)
                debug_log!(verbosity, "  Load instruction");
                self.execute_load_with_peripherals(instruction, memory, peripherals)
            }
            0x23 => {
                // Store instructions (SB, SH, SW)
                debug_log!(verbosity, "  Store instruction");
                self.execute_store_with_peripherals(instruction, memory, peripherals)
            }
            0x63 => {
                // Branch instructions (BEQ, BNE, BLT, BGE, BLTU, BGEU)
                debug_log!(verbosity, "  Branch instruction");
                self.execute_branch(instruction)
            }
            0x37 => {
                // LUI instruction
                debug_log!(verbosity, "  LUI instruction");
                self.execute_lui(instruction)
            }
            0x17 => {
                // AUIPC instruction
                debug_log!(verbosity, "  AUIPC instruction");
                self.execute_auipc(instruction)
            }
            0x6F => {
                // JAL instruction
                debug_log!(verbosity, "  JAL instruction");
                self.execute_jal(instruction)
            }
            0x67 => {
                // JALR instruction
                debug_log!(verbosity, "  JALR instruction");
                self.execute_jalr(instruction)
            }
            0x73 => {
                // System instructions (ECALL, EBREAK)
                debug_log!(verbosity, "  System instruction");
                self.execute_system(instruction)
            }
            0x2F => {
                // RV32A atomic instructions
                debug_log!(verbosity, "  Atomic instruction");
                self.execute_atomic_with_peripherals(instruction, memory, peripherals)
            }
            0x0F => {
                // FENCE instruction family (memory ordering)
                debug_log!(verbosity, "  FENCE instruction");
                let funct3 = (instruction >> 12) & 0x7;
                match funct3 {
                    0x0 => {
                        // FENCE - memory fence
                        // For our simple emulator, we'll treat it as a no-op
                        self.pc = self.pc.wrapping_add(4);
                        Ok(())
                    }
                    0x1 => {
                        // FENCE.I - instruction fence
                        // For our simple emulator, we'll treat it as a no-op
                        self.pc = self.pc.wrapping_add(4);
                        Ok(())
                    }
                    _ => Err(EmulatorError::UnsupportedInstruction),
                }
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
            0x2 => {
                // SLTI instruction
                self.execute_slti(rd, rs1, imm)
            }
            0x3 => {
                // SLTIU instruction
                self.execute_sltiu(rd, rs1, imm)
            }
            0x4 => {
                // XORI instruction
                self.execute_xori(rd, rs1, imm)
            }
            0x6 => {
                // ORI instruction
                self.execute_ori(rd, rs1, imm)
            }
            0x7 => {
                // ANDI instruction
                self.execute_andi(rd, rs1, imm)
            }
            0x1 => {
                // SLLI instruction
                if (imm as u32) & 0xFFE0 != 0 {
                    return Err(EmulatorError::UnsupportedInstruction);
                }
                self.execute_slli(rd, rs1, imm as u32 & 0x1F)
            }
            0x5 => {
                // SRLI/SRAI instruction (determined by bit 30)
                let is_srai = (instruction & 0x40000000) != 0;
                if (imm as u32) & 0xFFE0 != 0 && !is_srai {
                    return Err(EmulatorError::UnsupportedInstruction);
                }
                if is_srai {
                    self.execute_srai(rd, rs1, imm as u32 & 0x1F)
                } else {
                    self.execute_srli(rd, rs1, imm as u32 & 0x1F)
                }
            }
            _ => {
                // Unsupported funct3
                Err(EmulatorError::UnsupportedInstruction)
            }
        }
    }

    /// Execute R-type instructions
    fn execute_r_type(&mut self, instruction: u32) -> Result<()> {
        // Extract fields
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let rs2 = ((instruction >> 20) & 0x1F) as usize;
        let funct7 = (instruction >> 25) & 0x7F;

        match (funct7, funct3) {
            (0x00, 0x0) => {
                // ADD instruction
                self.execute_add(rd, rs1, rs2)
            }
            (0x20, 0x0) => {
                // SUB instruction
                self.execute_sub(rd, rs1, rs2)
            }
            (0x00, 0x1) => {
                // SLL instruction
                self.execute_sll(rd, rs1, rs2)
            }
            (0x00, 0x2) => {
                // SLT instruction
                self.execute_slt(rd, rs1, rs2)
            }
            (0x00, 0x3) => {
                // SLTU instruction
                self.execute_sltu(rd, rs1, rs2)
            }
            (0x00, 0x4) => {
                // XOR instruction
                self.execute_xor(rd, rs1, rs2)
            }
            (0x00, 0x5) => {
                // SRL instruction
                self.execute_srl(rd, rs1, rs2)
            }
            (0x20, 0x5) => {
                // SRA instruction
                self.execute_sra(rd, rs1, rs2)
            }
            (0x00, 0x6) => {
                // OR instruction
                self.execute_or(rd, rs1, rs2)
            }
            (0x00, 0x7) => {
                // AND instruction
                self.execute_and(rd, rs1, rs2)
            }
            (0x01, _) => {
                // RV32M extensions (MUL, DIV, etc.)
                self.execute_m_type(rd, rs1, rs2, funct3)
            }
            _ => {
                // Unsupported funct7/funct3 combination
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

    /// Execute I-type arithmetic and logical instructions
    pub fn execute_slti(&mut self, rd: usize, rs1: usize, imm: i32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1) as i32;
        let result = if rs1_value < imm { 1 } else { 0 };
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_sltiu(&mut self, rd: usize, rs1: usize, imm: i32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let imm_value = imm as u32;
        let result = if rs1_value < imm_value { 1 } else { 0 };
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_xori(&mut self, rd: usize, rs1: usize, imm: i32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let result = rs1_value ^ (imm as u32);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_ori(&mut self, rd: usize, rs1: usize, imm: i32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let result = rs1_value | (imm as u32);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_andi(&mut self, rd: usize, rs1: usize, imm: i32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let result = rs1_value & (imm as u32);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_slli(&mut self, rd: usize, rs1: usize, shamt: u32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || shamt >= 32 {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let result = rs1_value << shamt;
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_srli(&mut self, rd: usize, rs1: usize, shamt: u32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || shamt >= 32 {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let result = rs1_value >> shamt;
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_srai(&mut self, rd: usize, rs1: usize, shamt: u32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || shamt >= 32 {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1) as i32;
        let result = rs1_value >> shamt;
        self.write_register(rd, result as u32);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute R-type arithmetic and logical instructions
    pub fn execute_add(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);
        let result = rs1_value.wrapping_add(rs2_value);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_sub(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);
        let result = rs1_value.wrapping_sub(rs2_value);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_sll(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2) & 0x1F; // Only lower 5 bits used
        let result = rs1_value << rs2_value;
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_slt(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1) as i32;
        let rs2_value = self.read_register(rs2) as i32;
        let result = if rs1_value < rs2_value { 1 } else { 0 };
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_sltu(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);
        let result = if rs1_value < rs2_value { 1 } else { 0 };
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_xor(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);
        let result = rs1_value ^ rs2_value;
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_srl(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2) & 0x1F; // Only lower 5 bits used
        let result = rs1_value >> rs2_value;
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_sra(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1) as i32;
        let rs2_value = self.read_register(rs2) & 0x1F; // Only lower 5 bits used
        let result = rs1_value >> rs2_value;
        self.write_register(rd, result as u32);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_or(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);
        let result = rs1_value | rs2_value;
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    pub fn execute_and(&mut self, rd: usize, rs1: usize, rs2: usize) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);
        let result = rs1_value & rs2_value;
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute RV32M multiplication and division instructions
    fn execute_m_type(&mut self, rd: usize, rs1: usize, rs2: usize, funct3: u32) -> Result<()> {
        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);

        let result = match funct3 {
            0x0 => {
                // MUL
                let result = (rs1_value as u64).wrapping_mul(rs2_value as u64);
                result as u32
            }
            0x1 => {
                // MULH
                let result = (rs1_value as i32 as i64).wrapping_mul(rs2_value as i32 as i64);
                (result >> 32) as u32
            }
            0x2 => {
                // MULHSU
                let result = (rs1_value as i32 as i64).wrapping_mul(rs2_value as u64 as i64);
                (result >> 32) as u32
            }
            0x3 => {
                // MULHU
                let result = (rs1_value as u64).wrapping_mul(rs2_value as u64);
                (result >> 32) as u32
            }
            0x4 => {
                // DIV
                if rs2_value == 0 {
                    u32::MAX // Division by zero result
                } else if rs1_value == 0x80000000 && rs2_value == 0xFFFFFFFF {
                    0x80000000 // Overflow case
                } else {
                    ((rs1_value as i32) / (rs2_value as i32)) as u32
                }
            }
            0x5 => {
                // DIVU
                if rs2_value == 0 {
                    u32::MAX // Division by zero result
                } else {
                    rs1_value / rs2_value
                }
            }
            0x6 => {
                // REM
                if rs2_value == 0 {
                    rs1_value // Remainder of division by zero
                } else if rs1_value == 0x80000000 && rs2_value == 0xFFFFFFFF {
                    0 // Overflow case
                } else {
                    ((rs1_value as i32) % (rs2_value as i32)) as u32
                }
            }
            0x7 => {
                // REMU
                if rs2_value == 0 {
                    rs1_value // Remainder of division by zero
                } else {
                    rs1_value % rs2_value
                }
            }
            _ => return Err(EmulatorError::UnsupportedInstruction),
        };

        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute load instructions (LB, LH, LW, LBU, LHU)
    fn execute_load(&mut self, instruction: u32, memory: &mut Memory) -> Result<()> {
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let imm = (instruction as i32) >> 20; // Sign-extend immediate

        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let base_addr = self.read_register(rs1);
        let addr = base_addr.wrapping_add(imm as u32);

        match funct3 {
            0x0 => {
                // LB - Load byte (sign-extended)
                let value = memory.read_byte(addr)? as i8 as i32 as u32;
                self.write_register(rd, value);
            }
            0x1 => {
                // LH - Load halfword (sign-extended, supports misaligned access)
                let value = memory.read_halfword(addr)? as u32;
                let sign_extended = if value & 0x8000 != 0 {
                    value | 0xFFFF0000
                } else {
                    value
                };
                self.write_register(rd, sign_extended);
            }
            0x2 => {
                // LW - Load word
                let value = memory.read_word(addr)?;
                self.write_register(rd, value);
            }
            0x4 => {
                // LBU - Load byte unsigned
                let value = memory.read_byte(addr)? as u32;
                self.write_register(rd, value);
            }
            0x5 => {
                // LHU - Load halfword unsigned (supports misaligned access)
                let value = memory.read_halfword(addr)? as u32;
                self.write_register(rd, value);
            }
            _ => return Err(EmulatorError::UnsupportedInstruction),
        }

        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute store instructions (SB, SH, SW)
    fn execute_store(&mut self, instruction: u32, memory: &mut Memory) -> Result<()> {
        let imm_4_0 = (instruction >> 7) & 0x1F;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let rs2 = ((instruction >> 20) & 0x1F) as usize;
        let imm_11_5 = (instruction >> 25) & 0x7F;

        if rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        // Reconstruct 12-bit signed immediate
        let imm = ((imm_11_5 << 5) | imm_4_0) as i32;
        let imm = if imm & 0x800 != 0 {
            imm | 0xFFFFF000u32 as i32 // Sign extend
        } else {
            imm
        };

        let base_addr = self.read_register(rs1);
        let addr = base_addr.wrapping_add(imm as u32);
        let value = self.read_register(rs2);

        match funct3 {
            0x0 => {
                // SB - Store byte
                memory.write_byte(addr, value as u8)?;
            }
            0x1 => {
                // SH - Store halfword (supports misaligned access)
                memory.write_halfword(addr, value as u16)?;
            }
            0x2 => {
                // SW - Store word
                memory.write_word(addr, value)?;
            }
            _ => return Err(EmulatorError::UnsupportedInstruction),
        }

        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute load instructions with peripheral support
    fn execute_load_with_peripherals(
        &mut self,
        instruction: u32,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
    ) -> Result<()> {
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let imm = (instruction as i32) >> 20; // Sign-extend immediate

        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let base_addr = self.read_register(rs1);
        let addr = base_addr.wrapping_add(imm as u32);

        // Check if this is a peripheral address
        if peripherals.is_peripheral_address(addr) {
            // Only support word loads from peripherals for now
            match funct3 {
                0x2 => {
                    // LW - Load word from peripheral
                    let value = peripherals.read(addr)?;
                    self.write_register(rd, value);
                }
                _ => {
                    // For now, only support word access to peripherals
                    return Err(EmulatorError::UnsupportedInstruction);
                }
            }
        } else {
            // Normal memory access
            match funct3 {
                0x0 => {
                    // LB - Load byte (sign-extended)
                    let value = memory.read_byte(addr)? as i8 as i32 as u32;
                    self.write_register(rd, value);
                }
                0x1 => {
                    // LH - Load halfword (sign-extended, supports misaligned access)
                    let value = memory.read_halfword(addr)? as u32;
                    let sign_extended = if value & 0x8000 != 0 {
                        value | 0xFFFF0000
                    } else {
                        value
                    };
                    self.write_register(rd, sign_extended);
                }
                0x2 => {
                    // LW - Load word
                    let value = memory.read_word(addr)?;
                    self.write_register(rd, value);
                }
                0x4 => {
                    // LBU - Load byte unsigned
                    let value = memory.read_byte(addr)? as u32;
                    self.write_register(rd, value);
                }
                0x5 => {
                    // LHU - Load halfword unsigned (supports misaligned access)
                    let value = memory.read_halfword(addr)? as u32;
                    self.write_register(rd, value);
                }
                _ => return Err(EmulatorError::UnsupportedInstruction),
            }
        }

        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute store instructions with peripheral support
    fn execute_store_with_peripherals(
        &mut self,
        instruction: u32,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
    ) -> Result<()> {
        let imm_4_0 = (instruction >> 7) & 0x1F;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let rs2 = ((instruction >> 20) & 0x1F) as usize;
        let imm_11_5 = (instruction >> 25) & 0x7F;

        if rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        // Reconstruct 12-bit signed immediate
        let imm = ((imm_11_5 << 5) | imm_4_0) as i32;
        let imm = if imm & 0x800 != 0 {
            imm | 0xFFFFF000u32 as i32 // Sign extend
        } else {
            imm
        };

        let base_addr = self.read_register(rs1);
        let addr = base_addr.wrapping_add(imm as u32);
        let value = self.read_register(rs2);

        // Check if this is a peripheral address
        if peripherals.is_peripheral_address(addr) {
            // Only support word stores to peripherals for now
            match funct3 {
                0x2 => {
                    // SW - Store word to peripheral
                    peripherals.write(addr, value)?;
                }
                _ => {
                    // For now, only support word access to peripherals
                    return Err(EmulatorError::UnsupportedInstruction);
                }
            }
        } else {
            // Normal memory access
            match funct3 {
                0x0 => {
                    // SB - Store byte
                    memory.write_byte(addr, value as u8)?;
                }
                0x1 => {
                    // SH - Store halfword (supports misaligned access)
                    memory.write_halfword(addr, value as u16)?;
                }
                0x2 => {
                    // SW - Store word
                    memory.write_word(addr, value)?;
                }
                _ => return Err(EmulatorError::UnsupportedInstruction),
            }
        }

        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute branch instructions (BEQ, BNE, BLT, BGE, BLTU, BGEU)
    fn execute_branch(&mut self, instruction: u32) -> Result<()> {
        let imm_11 = (instruction >> 7) & 0x1;
        let imm_4_1 = (instruction >> 8) & 0xF;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let rs2 = ((instruction >> 20) & 0x1F) as usize;
        let imm_10_5 = (instruction >> 25) & 0x3F;
        let imm_12 = (instruction >> 31) & 0x1;

        if rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        // Reconstruct 13-bit signed branch offset (bit 0 is always 0)
        let imm = (imm_12 << 12) | (imm_11 << 11) | (imm_10_5 << 5) | (imm_4_1 << 1);
        let offset = if imm & 0x1000 != 0 {
            (imm | 0xFFFFE000) as i32 // Sign extend
        } else {
            imm as i32
        };

        let rs1_value = self.read_register(rs1);
        let rs2_value = self.read_register(rs2);

        let branch_taken = match funct3 {
            0x0 => rs1_value == rs2_value,                   // BEQ
            0x1 => rs1_value != rs2_value,                   // BNE
            0x4 => (rs1_value as i32) < (rs2_value as i32),  // BLT
            0x5 => (rs1_value as i32) >= (rs2_value as i32), // BGE
            0x6 => rs1_value < rs2_value,                    // BLTU
            0x7 => rs1_value >= rs2_value,                   // BGEU
            _ => return Err(EmulatorError::UnsupportedInstruction),
        };

        if branch_taken {
            self.pc = self.pc.wrapping_add(offset as u32);
        } else {
            self.pc = self.pc.wrapping_add(4);
        }

        Ok(())
    }

    /// Execute LUI instruction (Load Upper Immediate)
    fn execute_lui(&mut self, instruction: u32) -> Result<()> {
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let imm = instruction & 0xFFFFF000; // Upper 20 bits

        if rd >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        self.write_register(rd, imm);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute AUIPC instruction (Add Upper Immediate to PC)
    fn execute_auipc(&mut self, instruction: u32) -> Result<()> {
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let imm = instruction & 0xFFFFF000; // Upper 20 bits

        if rd >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let result = self.pc.wrapping_add(imm);
        self.write_register(rd, result);
        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute JAL instruction (Jump and Link)
    fn execute_jal(&mut self, instruction: u32) -> Result<()> {
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let imm_19_12 = (instruction >> 12) & 0xFF;
        let imm_11 = (instruction >> 20) & 0x1;
        let imm_10_1 = (instruction >> 21) & 0x3FF;
        let imm_20 = (instruction >> 31) & 0x1;

        if rd >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        // Reconstruct 21-bit signed jump offset (bit 0 is always 0)
        let imm = (imm_20 << 20) | (imm_19_12 << 12) | (imm_11 << 11) | (imm_10_1 << 1);
        let offset = if imm & 0x100000 != 0 {
            (imm | 0xFFE00000) as i32 // Sign extend
        } else {
            imm as i32
        };

        // Store return address (PC + 4)
        self.write_register(rd, self.pc.wrapping_add(4));

        // Jump to target
        self.pc = self.pc.wrapping_add(offset as u32);
        Ok(())
    }

    /// Execute JALR instruction (Jump and Link Register)
    fn execute_jalr(&mut self, instruction: u32) -> Result<()> {
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let imm = (instruction as i32) >> 20; // Sign-extend immediate

        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || funct3 != 0 {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let base_addr = self.read_register(rs1);
        let target = (base_addr.wrapping_add(imm as u32)) & !1; // Clear LSB

        // Store return address (PC + 4)
        self.write_register(rd, self.pc.wrapping_add(4));

        // Jump to target
        self.pc = target;
        Ok(())
    }

    /// Execute system instructions (ECALL, EBREAK, CSR operations)
    fn execute_system(&mut self, instruction: u32) -> Result<()> {
        let funct3 = (instruction >> 12) & 0x7;
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let csr = ((instruction >> 20) & 0xFFF) as u16;

        match funct3 {
            0x0 => {
                // ECALL/EBREAK/MRET
                let funct12 = instruction >> 20;
                match funct12 {
                    0x000 => {
                        // ECALL - Environment call
                        // This terminates execution for riscv-tests
                        Err(EmulatorError::EcallTermination)
                    }
                    0x001 => {
                        // EBREAK - Environment break
                        Err(EmulatorError::UnsupportedInstruction)
                    }
                    0x302 => {
                        // MRET - Machine return
                        // For our simple emulator, we'll just treat it as a no-op and continue
                        // In a real implementation, this would restore machine-mode state
                        self.pc = self.pc.wrapping_add(4);
                        Ok(())
                    }
                    _ => Err(EmulatorError::UnsupportedInstruction),
                }
            }
            0x1 => {
                // CSRRW - CSR Read/Write
                let old_value = self.read_csr(csr);
                if rd != 0 {
                    // Only read old value if rd is non-zero
                    self.write_register(rd, old_value);
                }
                let new_value = self.read_register(rs1);
                self.write_csr(csr, new_value);
                self.pc = self.pc.wrapping_add(4);
                Ok(())
            }
            0x2 => {
                // CSRRS - CSR Read and Set bits
                let old_value = self.read_csr(csr);
                if rs1 != 0 {
                    // Only write if rs1 is non-zero
                    let mask = self.read_register(rs1);
                    let new_value = old_value | mask;
                    self.write_csr(csr, new_value);
                }
                self.write_register(rd, old_value);
                self.pc = self.pc.wrapping_add(4);
                Ok(())
            }
            0x3 => {
                // CSRRC - CSR Read and Clear bits
                let old_value = self.read_csr(csr);
                if rs1 != 0 {
                    // Only write if rs1 is non-zero
                    let mask = self.read_register(rs1);
                    let new_value = old_value & !mask;
                    self.write_csr(csr, new_value);
                }
                self.write_register(rd, old_value);
                self.pc = self.pc.wrapping_add(4);
                Ok(())
            }
            0x5 => {
                // CSRRWI - CSR Read/Write Immediate
                let old_value = self.read_csr(csr);
                if rd != 0 {
                    // Only read old value if rd is non-zero
                    self.write_register(rd, old_value);
                }
                let imm = rs1 as u32; // rs1 field contains immediate value (zero-extended)
                self.write_csr(csr, imm);
                self.pc = self.pc.wrapping_add(4);
                Ok(())
            }
            0x6 => {
                // CSRRSI - CSR Read and Set bits Immediate
                let old_value = self.read_csr(csr);
                let imm = rs1 as u32; // rs1 field contains immediate value (zero-extended)
                if imm != 0 {
                    // Only write if immediate is non-zero
                    let new_value = old_value | imm;
                    self.write_csr(csr, new_value);
                }
                self.write_register(rd, old_value);
                self.pc = self.pc.wrapping_add(4);
                Ok(())
            }
            0x7 => {
                // CSRRCI - CSR Read and Clear bits Immediate
                let old_value = self.read_csr(csr);
                let imm = rs1 as u32; // rs1 field contains immediate value (zero-extended)
                if imm != 0 {
                    // Only write if immediate is non-zero
                    let new_value = old_value & !imm;
                    self.write_csr(csr, new_value);
                }
                self.write_register(rd, old_value);
                self.pc = self.pc.wrapping_add(4);
                Ok(())
            }
            _ => Err(EmulatorError::UnsupportedInstruction),
        }
    }

    /// Execute RV32A atomic instructions
    fn execute_atomic(&mut self, instruction: u32, memory: &mut Memory) -> Result<()> {
        let rd = ((instruction >> 7) & 0x1F) as usize;
        let funct3 = (instruction >> 12) & 0x7;
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        let rs2 = ((instruction >> 20) & 0x1F) as usize;
        let aq = (instruction >> 26) & 0x1;
        let rl = (instruction >> 25) & 0x1;
        let funct5 = (instruction >> 27) & 0x1F;

        if rd >= NUM_REGISTERS || rs1 >= NUM_REGISTERS || rs2 >= NUM_REGISTERS || funct3 != 0x2 {
            return Err(EmulatorError::UnsupportedInstruction);
        }

        let addr = self.read_register(rs1);

        // For this implementation, we'll ignore the aq/rl bits for simplicity
        let _ = (aq, rl);

        match funct5 {
            0x02 => {
                // LR.W - Load Reserved Word
                let value = memory.read_word(addr)?;
                self.write_register(rd, value);
                // TODO: Set reservation on address for SC.W
            }
            0x03 => {
                // SC.W - Store Conditional Word
                let value = self.read_register(rs2);
                memory.write_word(addr, value)?;
                // For simplicity, always succeed (return 0)
                self.write_register(rd, 0);
            }
            0x01 => {
                // AMOSWAP.W
                let old_value = memory.read_word(addr)?;
                let new_value = self.read_register(rs2);
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x00 => {
                // AMOADD.W
                let old_value = memory.read_word(addr)?;
                let new_value = old_value.wrapping_add(self.read_register(rs2));
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x04 => {
                // AMOXOR.W
                let old_value = memory.read_word(addr)?;
                let new_value = old_value ^ self.read_register(rs2);
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x0C => {
                // AMOAND.W
                let old_value = memory.read_word(addr)?;
                let new_value = old_value & self.read_register(rs2);
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x08 => {
                // AMOOR.W
                let old_value = memory.read_word(addr)?;
                let new_value = old_value | self.read_register(rs2);
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x10 => {
                // AMOMIN.W
                let old_value = memory.read_word(addr)?;
                let rs2_value = self.read_register(rs2);
                let new_value = if (old_value as i32) < (rs2_value as i32) {
                    old_value
                } else {
                    rs2_value
                };
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x14 => {
                // AMOMAX.W
                let old_value = memory.read_word(addr)?;
                let rs2_value = self.read_register(rs2);
                let new_value = if (old_value as i32) > (rs2_value as i32) {
                    old_value
                } else {
                    rs2_value
                };
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x18 => {
                // AMOMINU.W
                let old_value = memory.read_word(addr)?;
                let rs2_value = self.read_register(rs2);
                let new_value = if old_value < rs2_value {
                    old_value
                } else {
                    rs2_value
                };
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            0x1C => {
                // AMOMAXU.W
                let old_value = memory.read_word(addr)?;
                let rs2_value = self.read_register(rs2);
                let new_value = if old_value > rs2_value {
                    old_value
                } else {
                    rs2_value
                };
                memory.write_word(addr, new_value)?;
                self.write_register(rd, old_value);
            }
            _ => return Err(EmulatorError::UnsupportedInstruction),
        }

        self.pc = self.pc.wrapping_add(4);
        Ok(())
    }

    /// Execute atomic instructions with peripheral support
    fn execute_atomic_with_peripherals(
        &mut self,
        instruction: u32,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
    ) -> Result<()> {
        // For peripherals, we'll only allow normal memory atomic operations for now
        // Peripheral addresses don't support atomic operations in this implementation
        let rs1 = ((instruction >> 15) & 0x1F) as usize;
        if rs1 >= NUM_REGISTERS {
            return Err(EmulatorError::UnsupportedInstruction);
        }
        
        let addr = self.read_register(rs1);
        if peripherals.is_peripheral_address(addr) {
            // Atomic operations on peripherals not supported
            return Err(EmulatorError::UnsupportedInstruction);
        }
        
        // Use normal atomic implementation for memory addresses
        self.execute_atomic(instruction, memory)
    }

    /// Run the CPU until it encounters an error or reaches a halt condition
    pub fn run(&mut self, memory: &mut Memory, max_instructions: Option<u32>) -> Result<u32> {
        self.run_with_verbosity(memory, max_instructions, 0)
    }

    /// Run the CPU with verbose output until it encounters an error or reaches a halt condition
    pub fn run_with_verbosity(
        &mut self,
        memory: &mut Memory,
        max_instructions: Option<u32>,
        verbosity: u8,
    ) -> Result<u32> {
        let mut executed_instructions = 0;

        debug_log!(
            verbosity,
            "=== Starting CPU execution (verbose level {verbosity}) ==="
        );
        if let Some(limit) = max_instructions {
            debug_log!(verbosity, "Instruction limit: {limit}");
        }
        debug_log!(verbosity, "");

        loop {
            // Check instruction limit
            if let Some(max) = max_instructions {
                if executed_instructions >= max {
                    info_log!(verbosity, "Instruction limit ({max}) reached");
                    break;
                }
            }

            // Verbose output for cycle-by-cycle execution
            info_log!(
                verbosity,
                "Cycle {}: PC=0x{:08x}",
                executed_instructions + 1,
                self.pc
            );
            if verbosity >= 3 {
                // Show instruction being executed
                if let Ok(instruction) = memory.read_word(self.pc) {
                    debug_log!(verbosity, "  Instruction: 0x{instruction:08x}");
                    // Show some key registers before execution
                    debug_log!(
                        verbosity,
                        "  Before: x1=0x{:08x} x2=0x{:08x} x3=0x{:08x} x10=0x{:08x}",
                        self.read_register(1),
                        self.read_register(2),
                        self.read_register(3),
                        self.read_register(10)
                    );
                }
            }

            // Execute one instruction
            match self.step_with_verbosity(memory, verbosity) {
                Ok(()) => {
                    executed_instructions += 1;
                    debug_log!(
                        verbosity,
                        "  After:  x1=0x{:08x} x2=0x{:08x} x3=0x{:08x} x10=0x{:08x}",
                        self.read_register(1),
                        self.read_register(2),
                        self.read_register(3),
                        self.read_register(10)
                    );
                    debug_log!(verbosity, "");
                }
                Err(EmulatorError::UnsupportedInstruction) => {
                    basic_log!(
                        verbosity,
                        "Unsupported instruction at PC: 0x{:08x}",
                        self.pc
                    );
                    break;
                }
                Err(EmulatorError::EcallTermination) => {
                    // Normal termination via ECALL - this is expected in riscv-tests
                    executed_instructions += 1;
                    info_log!(verbosity, "ECALL termination at PC: 0x{:08x}", self.pc);
                    break;
                }
                Err(e) => {
                    basic_log!(verbosity, "Error at PC: 0x{:08x}: {e}", self.pc);
                    return Err(e);
                }
            }
        }

        debug_log!(verbosity, "=== CPU execution completed ===");
        debug_log!(
            verbosity,
            "Total instructions executed: {executed_instructions}"
        );

        Ok(executed_instructions)
    }

    /// Run the CPU with peripheral support until it encounters an error or reaches a halt condition
    pub fn run_with_peripherals(
        &mut self,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
        max_instructions: Option<u32>,
    ) -> Result<u32> {
        self.run_with_peripherals_and_verbosity(memory, peripherals, max_instructions, 0)
    }

    /// Run the CPU with peripheral and verbose support until it encounters an error or reaches a halt condition
    pub fn run_with_peripherals_and_verbosity(
        &mut self,
        memory: &mut Memory,
        peripherals: &mut crate::peripheral::PeripheralManager,
        max_instructions: Option<u32>,
        verbosity: u8,
    ) -> Result<u32> {
        let mut executed_instructions = 0;

        debug_log!(
            verbosity,
            "=== Starting CPU execution with peripherals (verbose level {verbosity}) ==="
        );
        if let Some(limit) = max_instructions {
            debug_log!(verbosity, "Instruction limit: {limit}");
        }
        debug_log!(verbosity, "");

        loop {
            // Check instruction limit
            if let Some(max) = max_instructions {
                if executed_instructions >= max {
                    info_log!(verbosity, "Instruction limit ({max}) reached");
                    break;
                }
            }

            // Verbose output for cycle-by-cycle execution
            info_log!(
                verbosity,
                "Cycle {}: PC=0x{:08x}",
                executed_instructions + 1,
                self.pc
            );

            // Execute one instruction
            match self.step_with_peripherals_and_verbosity(memory, peripherals, verbosity) {
                Ok(()) => {
                    executed_instructions += 1;
                }
                Err(EmulatorError::EcallTermination) => {
                    info_log!(verbosity, "ECALL termination detected");
                    break;
                }
                Err(e) => return Err(e),
            }
        }

        debug_log!(verbosity, "=== CPU execution completed ===");
        debug_log!(
            verbosity,
            "Total instructions executed: {executed_instructions}"
        );

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

    #[test]
    fn test_i_type_instructions() {
        let mut cpu = Cpu::new();

        // Test SLTI
        cpu.write_register(1, 10);
        cpu.execute_slti(2, 1, 15).unwrap();
        assert_eq!(cpu.read_register(2), 1); // 10 < 15
        cpu.execute_slti(3, 1, 5).unwrap();
        assert_eq!(cpu.read_register(3), 0); // 10 >= 5

        // Test SLTIU
        cpu.write_register(4, 10);
        cpu.execute_sltiu(5, 4, 15).unwrap();
        assert_eq!(cpu.read_register(5), 1); // 10 < 15
        cpu.execute_sltiu(6, 4, 5).unwrap();
        assert_eq!(cpu.read_register(6), 0); // 10 >= 5

        // Test XORI
        cpu.write_register(7, 0b1010);
        cpu.execute_xori(8, 7, 0b1100).unwrap();
        assert_eq!(cpu.read_register(8), 0b0110); // 1010 XOR 1100 = 0110

        // Test ORI
        cpu.write_register(9, 0b1010);
        cpu.execute_ori(10, 9, 0b1100).unwrap();
        assert_eq!(cpu.read_register(10), 0b1110); // 1010 OR 1100 = 1110

        // Test ANDI
        cpu.write_register(11, 0b1010);
        cpu.execute_andi(12, 11, 0b1100).unwrap();
        assert_eq!(cpu.read_register(12), 0b1000); // 1010 AND 1100 = 1000

        // Test SLLI
        cpu.write_register(13, 5);
        cpu.execute_slli(14, 13, 2).unwrap();
        assert_eq!(cpu.read_register(14), 20); // 5 << 2 = 20

        // Test SRLI
        cpu.write_register(15, 20);
        cpu.execute_srli(16, 15, 2).unwrap();
        assert_eq!(cpu.read_register(16), 5); // 20 >> 2 = 5

        // Test SRAI
        cpu.write_register(17, 0xFFFFFFFC_u32); // -4 in two's complement
        cpu.execute_srai(18, 17, 1).unwrap();
        assert_eq!(cpu.read_register(18), 0xFFFFFFFE_u32); // -4 >> 1 = -2 (arithmetic shift)
    }

    #[test]
    fn test_r_type_instructions() {
        let mut cpu = Cpu::new();

        // Test ADD
        cpu.write_register(1, 10);
        cpu.write_register(2, 5);
        cpu.execute_add(3, 1, 2).unwrap();
        assert_eq!(cpu.read_register(3), 15);

        // Test SUB
        cpu.execute_sub(4, 1, 2).unwrap();
        assert_eq!(cpu.read_register(4), 5);

        // Test SLL
        cpu.write_register(5, 3);
        cpu.execute_sll(6, 1, 5).unwrap(); // 10 << 3 = 80
        assert_eq!(cpu.read_register(6), 80);

        // Test SLT
        cpu.execute_slt(7, 2, 1).unwrap(); // 5 < 10
        assert_eq!(cpu.read_register(7), 1);

        // Test SLTU
        cpu.execute_sltu(8, 1, 2).unwrap(); // 10 < 5 (unsigned)
        assert_eq!(cpu.read_register(8), 0);

        // Test XOR
        cpu.write_register(9, 0b1010);
        cpu.write_register(10, 0b1100);
        cpu.execute_xor(11, 9, 10).unwrap();
        assert_eq!(cpu.read_register(11), 0b0110);

        // Test SRL
        cpu.write_register(12, 80);
        cpu.execute_srl(13, 12, 5).unwrap(); // 80 >> 3 = 10
        assert_eq!(cpu.read_register(13), 10);

        // Test SRA (arithmetic right shift)
        cpu.write_register(14, 0xFFFFFFE0_u32); // -32 in two's complement
        cpu.execute_sra(15, 14, 5).unwrap(); // -32 >> 3 = -4
        assert_eq!(cpu.read_register(15), 0xFFFFFFFC_u32);

        // Test OR
        cpu.execute_or(16, 9, 10).unwrap(); // 1010 OR 1100 = 1110
        assert_eq!(cpu.read_register(16), 0b1110);

        // Test AND
        cpu.execute_and(17, 9, 10).unwrap(); // 1010 AND 1100 = 1000
        assert_eq!(cpu.read_register(17), 0b1000);
    }

    #[test]
    fn test_m_type_instructions() {
        let mut cpu = Cpu::new();

        // Test MUL
        cpu.write_register(1, 6);
        cpu.write_register(2, 7);
        cpu.execute_m_type(3, 1, 2, 0x0).unwrap(); // MUL
        assert_eq!(cpu.read_register(3), 42);

        // Test MULH (signed multiplication high)
        cpu.write_register(4, 0x80000000); // Large number
        cpu.write_register(5, 2);
        cpu.execute_m_type(6, 4, 5, 0x1).unwrap(); // MULH
                                                   // Should get upper 32 bits of signed multiplication

        // Test DIV
        cpu.write_register(7, 42);
        cpu.write_register(8, 6);
        cpu.execute_m_type(9, 7, 8, 0x4).unwrap(); // DIV
        assert_eq!(cpu.read_register(9), 7);

        // Test DIVU
        cpu.execute_m_type(10, 7, 8, 0x5).unwrap(); // DIVU
        assert_eq!(cpu.read_register(10), 7);

        // Test REM
        cpu.write_register(11, 43);
        cpu.execute_m_type(12, 11, 8, 0x6).unwrap(); // REM
        assert_eq!(cpu.read_register(12), 1); // 43 % 6 = 1

        // Test REMU
        cpu.execute_m_type(13, 11, 8, 0x7).unwrap(); // REMU
        assert_eq!(cpu.read_register(13), 1); // 43 % 6 = 1

        // Test division by zero
        cpu.write_register(14, 0);
        cpu.execute_m_type(15, 7, 14, 0x4).unwrap(); // DIV by zero
        assert_eq!(cpu.read_register(15), u32::MAX); // Should return -1
    }

    #[test]
    fn test_load_store_instructions() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let base_addr = memory.base_address();

        // Set up test data in memory
        memory.write_word(base_addr, 0x12345678).unwrap();
        memory.write_word(base_addr + 4, 0xDEADBEEF).unwrap();

        // Test LW (Load Word)
        let lw_instruction = (0 << 20) | (0 << 15) | (0x2 << 12) | (1 << 7) | 0x03; // lw x1, 0(x0)
        cpu.write_register(0, base_addr); // This won't actually change x0, but for the test we set the base
        cpu.execute_load(lw_instruction, &mut memory).unwrap();
        // Since x0 is always 0, we need to manually set up the test differently

        // Better test: use a non-zero base register
        cpu.write_register(2, base_addr);
        let lw_instruction = (0 << 20) | (2 << 15) | (0x2 << 12) | (1 << 7) | 0x03; // lw x1, 0(x2)
        cpu.execute_load(lw_instruction, &mut memory).unwrap();
        assert_eq!(cpu.read_register(1), 0x12345678);

        // Test LB (Load Byte signed)
        let lb_instruction = (0 << 20) | (2 << 15) | (0x0 << 12) | (3 << 7) | 0x03; // lb x3, 0(x2)
        cpu.execute_load(lb_instruction, &mut memory).unwrap();
        assert_eq!(cpu.read_register(3), 0x78); // LSB of 0x12345678

        // Test LBU (Load Byte unsigned)
        let lbu_instruction = (0 << 20) | (2 << 15) | (0x4 << 12) | (4 << 7) | 0x03; // lbu x4, 0(x2)
        cpu.execute_load(lbu_instruction, &mut memory).unwrap();
        assert_eq!(cpu.read_register(4), 0x78);

        // Test SW (Store Word)
        cpu.write_register(5, 0xCAFEBABE);
        let sw_instruction = (0 << 25) | (5 << 20) | (2 << 15) | (0x2 << 12) | (8 << 7) | 0x23; // sw x5, 8(x2)
        cpu.execute_store(sw_instruction, &mut memory).unwrap();
        assert_eq!(memory.read_word(base_addr + 8).unwrap(), 0xCAFEBABE);

        // Test SB (Store Byte)
        cpu.write_register(6, 0xAB);
        let sb_instruction = (0 << 25) | (6 << 20) | (2 << 15) | (0x0 << 12) | (12 << 7) | 0x23; // sb x6, 12(x2)
        cpu.execute_store(sb_instruction, &mut memory).unwrap();
        assert_eq!(memory.read_byte(base_addr + 12).unwrap(), 0xAB);
    }

    #[test]
    fn test_branch_instructions() {
        let mut cpu = Cpu::new();

        cpu.pc = 1000;
        cpu.write_register(1, 10);
        cpu.write_register(2, 10);

        // Test BEQ (Branch if Equal) - should branch with offset 8
        // For offset 8 = 0b1000, we need:
        // imm[4:1] = 0b100 = 4 (because offset 8 >> 1 = 4)
        // imm[10:5] = 0, imm[11] = 0, imm[12] = 0
        let imm_12 = 0u32;
        let imm_11 = 0u32;
        let imm_10_5 = 0u32;
        let imm_4_1 = 0b0100u32; // 4 in binary
        let beq_instruction = (imm_12 << 31)
            | (imm_10_5 << 25)
            | (2 << 20)
            | (1 << 15)
            | (0x0 << 12)
            | (imm_4_1 << 8)
            | (imm_11 << 7)
            | 0x63;
        cpu.execute_branch(beq_instruction).unwrap();
        assert_eq!(cpu.pc, 1008); // 1000 + 8

        // Test BNE (Branch if Not Equal) - should not branch since registers are equal
        cpu.pc = 1000;
        let bne_instruction = (imm_12 << 31)
            | (imm_10_5 << 25)
            | (2 << 20)
            | (1 << 15)
            | (0x1 << 12)
            | (imm_4_1 << 8)
            | (imm_11 << 7)
            | 0x63;
        cpu.execute_branch(bne_instruction).unwrap();
        assert_eq!(cpu.pc, 1004); // 1000 + 4 (no branch, just increment)

        // Test BLT (Branch if Less Than)
        cpu.pc = 1000;
        cpu.write_register(3, 5);
        // blt x3, x1, 8 (5 < 10, should branch)
        let blt_instruction = (imm_12 << 31)
            | (imm_10_5 << 25)
            | (1 << 20)
            | (3 << 15)
            | (0x4 << 12)
            | (imm_4_1 << 8)
            | (imm_11 << 7)
            | 0x63;
        cpu.execute_branch(blt_instruction).unwrap();
        assert_eq!(cpu.pc, 1008); // 1000 + 8 (branch taken)
    }

    #[test]
    fn test_upper_immediate_instructions() {
        let mut cpu = Cpu::new();

        // Test LUI (Load Upper Immediate)
        let lui_instruction = (0x12345 << 12) | (1 << 7) | 0x37; // lui x1, 0x12345
        cpu.execute_lui(lui_instruction).unwrap();
        assert_eq!(cpu.read_register(1), 0x12345000);

        // Test AUIPC (Add Upper Immediate to PC)
        cpu.pc = 0x1000;
        let auipc_instruction = (0x12345 << 12) | (2 << 7) | 0x17; // auipc x2, 0x12345
        cpu.execute_auipc(auipc_instruction).unwrap();
        assert_eq!(cpu.read_register(2), 0x1000 + 0x12345000);
        assert_eq!(cpu.pc, 0x1004); // PC should be incremented
    }

    #[test]
    fn test_jump_instructions() {
        let mut cpu = Cpu::new();

        // Test JAL (Jump and Link)
        cpu.pc = 1000;
        // Create JAL instruction with offset 8
        // J-type format: imm[20|10:1|11|19:12] | rd | opcode
        // For offset 8: imm[20] = 0, imm[10:1] = 0000000100, imm[11] = 0, imm[19:12] = 00000000
        let imm_20 = 0u32;
        let imm_19_12 = 0u32;
        let imm_11 = 0u32;
        let imm_10_1 = 0b0000000100u32; // 8 >> 1 = 4
        let jal_instruction = (imm_20 << 31)
            | (imm_10_1 << 21)
            | (imm_11 << 20)
            | (imm_19_12 << 12)
            | (1 << 7)
            | 0x6F;
        cpu.execute_jal(jal_instruction).unwrap();
        assert_eq!(cpu.read_register(1), 1004); // Return address (PC + 4)
        assert_eq!(cpu.pc, 1008); // Jump target (1000 + 8)

        // Test JALR (Jump and Link Register)
        cpu.pc = 2000;
        cpu.write_register(2, 3000);
        let jalr_instruction = (4 << 20) | (2 << 15) | (0 << 12) | (3 << 7) | 0x67; // jalr x3, 4(x2)
        cpu.execute_jalr(jalr_instruction).unwrap();
        assert_eq!(cpu.read_register(3), 2004); // Return address
        assert_eq!(cpu.pc, 3004); // Jump target (3000 + 4)
    }

    #[test]
    fn test_atomic_instructions() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let base_addr = memory.base_address();

        // Initialize memory
        memory.write_word(base_addr, 100).unwrap();
        cpu.write_register(1, base_addr);

        // Test LR.W (Load Reserved Word)
        let lr_instruction = (0x02 << 27) | (0 << 20) | (1 << 15) | (0x2 << 12) | (2 << 7) | 0x2F;
        cpu.execute_atomic(lr_instruction, &mut memory).unwrap();
        assert_eq!(cpu.read_register(2), 100);

        // Test SC.W (Store Conditional Word)
        cpu.write_register(3, 200);
        let sc_instruction = (0x03 << 27) | (3 << 20) | (1 << 15) | (0x2 << 12) | (4 << 7) | 0x2F;
        cpu.execute_atomic(sc_instruction, &mut memory).unwrap();
        assert_eq!(cpu.read_register(4), 0); // Success
        assert_eq!(memory.read_word(base_addr).unwrap(), 200);

        // Test AMOSWAP.W
        cpu.write_register(5, 300);
        let amoswap_instruction =
            (0x01 << 27) | (5 << 20) | (1 << 15) | (0x2 << 12) | (6 << 7) | 0x2F;
        cpu.execute_atomic(amoswap_instruction, &mut memory)
            .unwrap();
        assert_eq!(cpu.read_register(6), 200); // Old value
        assert_eq!(memory.read_word(base_addr).unwrap(), 300); // New value

        // Test AMOADD.W
        cpu.write_register(7, 50);
        let amoadd_instruction =
            (0x00 << 27) | (7 << 20) | (1 << 15) | (0x2 << 12) | (8 << 7) | 0x2F;
        cpu.execute_atomic(amoadd_instruction, &mut memory).unwrap();
        assert_eq!(cpu.read_register(8), 300); // Old value
        assert_eq!(memory.read_word(base_addr).unwrap(), 350); // 300 + 50
    }

    #[test]
    fn test_csr_instructions() {
        let mut cpu = Cpu::new();

        // Test basic CSR read/write
        cpu.write_csr(0x300, 0x12345678); // mstatus
        assert_eq!(cpu.read_csr(0x300), 0x12345678);

        // Test CSRRW - should work as expected
        cpu.write_register(1, 0xABCDEF00);
        cpu.csrs.insert(0x301, 0x11111111);

        // CSRRW x2, 0x301, x1 - read 0x301 into x2, write x1 into 0x301
        let csrrw = (0x301 << 20) | (1 << 15) | (1 << 12) | (2 << 7) | 0x73;
        assert!(cpu.execute_system(csrrw).is_ok());
        assert_eq!(cpu.read_register(2), 0x11111111); // Old value of CSR
        assert_eq!(cpu.read_csr(0x301), 0xABCDEF00); // New value written

        // Test CSRRS with rs1=0 (should not write)
        let old_csr = cpu.read_csr(0x301);
        let csrrs_no_write = (0x301 << 20) | (0 << 15) | (2 << 12) | (3 << 7) | 0x73;
        assert!(cpu.execute_system(csrrs_no_write).is_ok());
        assert_eq!(cpu.read_csr(0x301), old_csr); // Should be unchanged
        assert_eq!(cpu.read_register(3), old_csr); // Should have read the value

        // Test CSRRS with rs1!=0 (should write)
        cpu.write_register(4, 0x0000F000);
        let csrrs_write = (0x301 << 20) | (4 << 15) | (2 << 12) | (5 << 7) | 0x73;
        assert!(cpu.execute_system(csrrs_write).is_ok());
        assert_eq!(cpu.read_register(5), old_csr); // Should have read old value
        assert_eq!(cpu.read_csr(0x301), old_csr | 0x0000F000); // Should have set bits
    }

    #[test]
    fn test_fence_instructions() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new();
        let base_addr = memory.base_address();

        cpu.pc = base_addr;

        // Test FENCE instruction (funct3=0)
        let fence_instruction = (0x0 << 12) | 0x0F;
        memory.write_word(cpu.pc, fence_instruction).unwrap();

        let old_pc = cpu.pc;
        cpu.step(&mut memory).unwrap();
        assert_eq!(cpu.pc, old_pc + 4); // Should advance PC

        // Test FENCE.I instruction (funct3=1)
        let fence_i_instruction = (0x1 << 12) | 0x0F;
        memory.write_word(cpu.pc, fence_i_instruction).unwrap();

        let old_pc = cpu.pc;
        cpu.step(&mut memory).unwrap();
        assert_eq!(cpu.pc, old_pc + 4); // Should advance PC
    }
}
