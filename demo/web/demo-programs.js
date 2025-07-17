// Simple test program generator for the RISC-V emulator demo
// This creates a basic binary that outputs "Hello, Life!" to the console

function generateTestProgram() {
    // Simple RISC-V program that outputs to console peripheral
    const instructions = [];
    
    // Base address for UART peripheral: 0x10000000
    const UART_BASE = 0x10000000;
    
    // li t0, 0x10000000  (load UART base address)
    // lui t0, 0x10000
    instructions.push(0x10000337); // lui t0, 0x10000
    
    // Message: "Hello, Life!\n"
    const message = "Hello, Conway's Game of Life!\n";
    
    for (let i = 0; i < message.length; i++) {
        const char = message.charCodeAt(i);
        
        // li t1, char  (load character)
        instructions.push(0x00000313 | (char << 20)); // addi t1, x0, char
        
        // sw t1, 0(t0)  (store to UART TX)
        instructions.push(0x00632023); // sw t1, 0(t0)
    }
    
    // Exit with ECALL
    // li a7, 93 (sys_exit)
    instructions.push(0x05d00893); // addi a7, x0, 93
    
    // li a0, 0 (exit code 0)
    instructions.push(0x00000513); // addi a0, x0, 0
    
    // ecall
    instructions.push(0x00000073); // ecall
    
    // Convert to little-endian byte array
    const binary = new Uint8Array(instructions.length * 4);
    for (let i = 0; i < instructions.length; i++) {
        const inst = instructions[i];
        binary[i * 4] = inst & 0xFF;
        binary[i * 4 + 1] = (inst >> 8) & 0xFF;
        binary[i * 4 + 2] = (inst >> 16) & 0xFF;
        binary[i * 4 + 3] = (inst >> 24) & 0xFF;
    }
    
    return binary;
}

// For the actual Game of Life, let's create a more sophisticated program
function generateLifeGameProgram() {
    // This would be a much more complex program
    // For now, we'll create a simple program that demonstrates the concept
    const instructions = [];
    
    const UART_BASE = 0x10000000;
    
    // Load UART base
    instructions.push(0x10000337); // lui t0, 0x10000
    
    // Simple game of life simulation output
    const pattern = [
        "=== Conway's Game of Life ===\n",
        "Generation 1:\n",
        "..........#..........\n",
        ".........#.#.........\n",
        ".........##..........\n",
        ".....................\n",
        "\nGeneration 2:\n",
        "........###..........\n",
        "........#.#..........\n",
        ".........##..........\n",
        ".....................\n",
        "\nGame completed!\n"
    ];
    
    for (const line of pattern) {
        for (let i = 0; i < line.length; i++) {
            const char = line.charCodeAt(i);
            
            // Load character and send to UART
            instructions.push(0x00000313 | (char << 20)); // addi t1, x0, char
            instructions.push(0x00632023); // sw t1, 0(t0)
            
            // Simple delay loop (very short for demo)
            instructions.push(0x00100393); // addi t2, x0, 1
            instructions.push(0xfff38393); // addi t2, t2, -1
            instructions.push(0xfe039ee3); // bnez t2, loop (previous instruction)
        }
    }
    
    // Exit
    instructions.push(0x05d00893); // addi a7, x0, 93 (sys_exit)
    instructions.push(0x00000513); // addi a0, x0, 0 (exit code 0)
    instructions.push(0x00000073); // ecall
    
    // Convert to byte array
    const binary = new Uint8Array(instructions.length * 4);
    for (let i = 0; i < instructions.length; i++) {
        const inst = instructions[i];
        binary[i * 4] = inst & 0xFF;
        binary[i * 4 + 1] = (inst >> 8) & 0xFF;
        binary[i * 4 + 2] = (inst >> 16) & 0xFF;
        binary[i * 4 + 3] = (inst >> 24) & 0xFF;
    }
    
    return binary;
}

// Export for use in the web demo
if (typeof window !== 'undefined') {
    window.generateTestProgram = generateTestProgram;
    window.generateLifeGameProgram = generateLifeGameProgram;
} else if (typeof module !== 'undefined') {
    module.exports = { generateTestProgram, generateLifeGameProgram };
}