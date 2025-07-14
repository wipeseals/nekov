#include <stdint.h>

// Simple entry point for bare-metal RISC-V
__attribute__((section(".text.entry")))
void _start() {
    // Set up stack pointer 
    __asm__ volatile("li sp, 0x80100000");
    
    // Call main
    extern int main();
    main();
    
    // If main returns, exit
    __asm__ volatile("li a7, 93");  // sys_exit
    __asm__ volatile("li a0, 0");   // exit code 0
    __asm__ volatile("ecall");
    
    // Infinite loop (shouldn't reach here)
    while(1);
}

// Memory-mapped UART for output
#define UART_BASE 0x10000000
#define UART_TX   (*(volatile uint32_t*)(UART_BASE + 0))

// Simple Conway's Game of Life for RISC-V
#define WIDTH 20
#define HEIGHT 10

// Two grids for double buffering
static char grid[HEIGHT][WIDTH];
static char next_grid[HEIGHT][WIDTH];

void uart_putc(char c) {
    UART_TX = (uint32_t)c;
}

void uart_puts(const char *str) {
    while (*str) {
        uart_putc(*str++);
    }
}

void clear_grid(char g[HEIGHT][WIDTH]) {
    for (int i = 0; i < HEIGHT; i++) {
        for (int j = 0; j < WIDTH; j++) {
            g[i][j] = 0;
        }
    }
}

void print_grid(char g[HEIGHT][WIDTH]) {
    uart_puts("\n=== Conway's Game of Life ===\n");
    for (int i = 0; i < HEIGHT; i++) {
        for (int j = 0; j < WIDTH; j++) {
            uart_putc(g[i][j] ? '#' : '.');
        }
        uart_putc('\n');
    }
    uart_puts("\n");
}

int count_neighbors(char g[HEIGHT][WIDTH], int row, int col) {
    int count = 0;
    for (int i = -1; i <= 1; i++) {
        for (int j = -1; j <= 1; j++) {
            if (i == 0 && j == 0) continue; // Skip self
            
            int r = row + i;
            int c = col + j;
            
            // Wrap around (toroidal topology)
            if (r < 0) r = HEIGHT - 1;
            if (r >= HEIGHT) r = 0;
            if (c < 0) c = WIDTH - 1;
            if (c >= WIDTH) c = 0;
            
            if (g[r][c]) count++;
        }
    }
    return count;
}

void update_grid() {
    for (int i = 0; i < HEIGHT; i++) {
        for (int j = 0; j < WIDTH; j++) {
            int neighbors = count_neighbors(grid, i, j);
            
            if (grid[i][j]) {
                // Cell is alive
                next_grid[i][j] = (neighbors == 2 || neighbors == 3) ? 1 : 0;
            } else {
                // Cell is dead
                next_grid[i][j] = (neighbors == 3) ? 1 : 0;
            }
        }
    }
    
    // Copy next_grid to grid
    for (int i = 0; i < HEIGHT; i++) {
        for (int j = 0; j < WIDTH; j++) {
            grid[i][j] = next_grid[i][j];
        }
    }
}

void init_glider() {
    clear_grid(grid);
    
    // Classic glider pattern
    grid[1][2] = 1;
    grid[2][3] = 1;
    grid[3][1] = 1;
    grid[3][2] = 1;
    grid[3][3] = 1;
}

// Simple delay function
void delay() {
    for (volatile int i = 0; i < 1000000; i++) {
        // Busy wait
    }
}

int main() {
    uart_puts("Starting Conway's Game of Life on RISC-V!\n");
    
    init_glider();
    
    for (int generation = 0; generation < 1000; generation++) {
        print_grid(grid);
        
        // Small delay to make it visible
        delay();
        
        update_grid();
    }
    
    uart_puts("Game of Life completed!\n");
    
    // Terminate program with ECALL
    __asm__ volatile("li a7, 93");  // sys_exit
    __asm__ volatile("li a0, 0");   // exit code 0
    __asm__ volatile("ecall");
    
    return 0;
}