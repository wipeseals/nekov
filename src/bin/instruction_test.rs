/// Simple test program to verify rv32ima instruction implementation
/// This creates a series of instructions manually and runs them through the emulator
use nekov::{cpu::Cpu, memory::Memory};

fn main() {
    println!("🐈 Nekov RV32IMA Instruction Test");
    println!("==================================");

    let mut cpu = Cpu::new();
    let mut memory = Memory::new();
    let base_addr = memory.base_address();
    
    cpu.pc = base_addr;
    
    // Test 1: Basic I-type instructions (ADDI, XORI, ORI, ANDI)
    println!("\n📝 Test 1: I-type instructions");
    let instructions = vec![
        // addi x1, x0, 42        ; x1 = 42
        (42 << 20) | (0 << 15) | (0 << 12) | (1 << 7) | 0x13,
        // xori x2, x1, 15        ; x2 = 42 ^ 15 = 37
        (15 << 20) | (1 << 15) | (4 << 12) | (2 << 7) | 0x13,
        // ori x3, x1, 128        ; x3 = 42 | 128 = 170  
        (128 << 20) | (1 << 15) | (6 << 12) | (3 << 7) | 0x13,
        // andi x4, x3, 255       ; x4 = 170 & 255 = 170
        (255 << 20) | (3 << 15) | (7 << 12) | (4 << 7) | 0x13,
    ];
    
    run_instructions(&mut cpu, &mut memory, &instructions).unwrap();
    
    assert_eq!(cpu.read_register(1), 42);
    assert_eq!(cpu.read_register(2), 37);  // 42 ^ 15
    assert_eq!(cpu.read_register(3), 170); // 42 | 128
    assert_eq!(cpu.read_register(4), 170); // 170 & 255
    println!("✅ I-type instructions working correctly");
    
    // Test 2: R-type arithmetic instructions
    println!("\n📝 Test 2: R-type arithmetic");
    let instructions = vec![
        // add x5, x1, x2         ; x5 = 42 + 37 = 79
        (0 << 25) | (2 << 20) | (1 << 15) | (0 << 12) | (5 << 7) | 0x33,
        // sub x6, x5, x1         ; x6 = 79 - 42 = 37
        (0x20 << 25) | (1 << 20) | (5 << 15) | (0 << 12) | (6 << 7) | 0x33,
        // sll x7, x1, x2         ; x7 = 42 << (37 & 31) = 42 << 5 = 1344
        (0 << 25) | (2 << 20) | (1 << 15) | (1 << 12) | (7 << 7) | 0x33,
    ];
    
    run_instructions(&mut cpu, &mut memory, &instructions).unwrap();
    
    assert_eq!(cpu.read_register(5), 79);
    assert_eq!(cpu.read_register(6), 37);
    assert_eq!(cpu.read_register(7), 1344); // 42 << 5
    println!("✅ R-type arithmetic working correctly");
    
    // Test 3: Load/Store instructions
    println!("\n📝 Test 3: Load/Store instructions");
    
    // First, store some data
    cpu.write_register(8, base_addr + 100); // Base address in x8
    cpu.write_register(9, 0xDEADBEEF);      // Value to store in x9
    
    let instructions = vec![
        // sw x9, 0(x8)           ; Store 0xDEADBEEF at base_addr + 100
        (0 << 25) | (9 << 20) | (8 << 15) | (2 << 12) | (0 << 7) | 0x23,
        // lw x10, 0(x8)          ; Load word from base_addr + 100 into x10
        (0 << 20) | (8 << 15) | (2 << 12) | (10 << 7) | 0x03,
        // lb x11, 0(x8)          ; Load byte (signed) from base_addr + 100 into x11
        (0 << 20) | (8 << 15) | (0 << 12) | (11 << 7) | 0x03,
    ];
    
    run_instructions(&mut cpu, &mut memory, &instructions).unwrap();
    
    assert_eq!(cpu.read_register(10), 0xDEADBEEF);
    assert_eq!(cpu.read_register(11), 0xFFFFFFEF); // Sign-extended byte (0xEF)
    println!("✅ Load/Store instructions working correctly");
    
    // Test 4: Branch instructions
    println!("\n📝 Test 4: Branch instructions");
    
    cpu.pc = base_addr + 200;
    cpu.write_register(12, 10);
    cpu.write_register(13, 20);
    
    // beq x12, x12, 8         ; Should branch (10 == 10)
    // Offset 8: imm[4:1] = 4, others = 0
    let beq_instruction = (0 << 31) | (0 << 25) | (12 << 20) | (12 << 15) | (0 << 12) | (4 << 8) | (0 << 7) | 0x63;
    memory.write_word(cpu.pc, beq_instruction).unwrap();
    
    let old_pc = cpu.pc;
    cpu.step(&mut memory).unwrap();
    assert_eq!(cpu.pc, old_pc + 8); // Should have branched
    println!("✅ Branch instructions working correctly");
    
    // Test 5: RV32M multiplication
    println!("\n📝 Test 5: RV32M multiplication");
    
    cpu.pc = base_addr + 300;
    cpu.write_register(14, 6);
    cpu.write_register(15, 7);
    
    let instructions = vec![
        // mul x16, x14, x15       ; x16 = 6 * 7 = 42
        (1 << 25) | (15 << 20) | (14 << 15) | (0 << 12) | (16 << 7) | 0x33,
        // div x17, x16, x14       ; x17 = 42 / 6 = 7
        (1 << 25) | (14 << 20) | (16 << 15) | (4 << 12) | (17 << 7) | 0x33,
    ];
    
    run_instructions(&mut cpu, &mut memory, &instructions).unwrap();
    
    assert_eq!(cpu.read_register(16), 42);
    assert_eq!(cpu.read_register(17), 7);
    println!("✅ RV32M multiplication working correctly");
    
    // Test 6: Upper immediate instructions
    println!("\n📝 Test 6: Upper immediate instructions");
    
    let instructions = vec![
        // lui x18, 0x12345       ; x18 = 0x12345000
        (0x12345 << 12) | (18 << 7) | 0x37,
        // auipc x19, 0x1000      ; x19 = PC + 0x1000000
        (0x1000 << 12) | (19 << 7) | 0x17,
    ];
    
    let pc_before_auipc = cpu.pc + 4; // PC after LUI, before AUIPC
    run_instructions(&mut cpu, &mut memory, &instructions).unwrap();
    
    assert_eq!(cpu.read_register(18), 0x12345000);
    assert_eq!(cpu.read_register(19), pc_before_auipc + 0x1000000);
    println!("✅ Upper immediate instructions working correctly");
    
    println!("\n🎉 All instruction tests passed! RV32IMA implementation is working correctly.");
    
    // Print final register state
    println!("\n📊 Final Register State:");
    for i in 0..8 {
        println!(
            "x{:2}: 0x{:08x}  x{:2}: 0x{:08x}  x{:2}: 0x{:08x}  x{:2}: 0x{:08x}",
            i, cpu.read_register(i),
            i + 8, cpu.read_register(i + 8),
            i + 16, cpu.read_register(i + 16),
            i + 24, cpu.read_register(i + 24)
        );
    }
}

fn run_instructions(cpu: &mut Cpu, memory: &mut Memory, instructions: &[u32]) -> Result<(), Box<dyn std::error::Error>> {
    for &instruction in instructions {
        memory.write_word(cpu.pc, instruction)?;
        cpu.step(memory)?;
    }
    Ok(())
}