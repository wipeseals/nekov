/* Linker script for bare-metal RISC-V program */

ENTRY(_start)

MEMORY
{
    RAM : ORIGIN = 0x80000000, LENGTH = 16M
}

SECTIONS
{
    . = 0x80000000;
    
    .text : {
        *(.text.entry)
        *(.text)
        *(.text.*)
    } > RAM
    
    .rodata : {
        *(.rodata)
        *(.rodata.*)
    } > RAM
    
    .data : {
        *(.data)
        *(.data.*)
    } > RAM
    
    .bss : {
        *(.bss)
        *(.bss.*)
        *(COMMON)
    } > RAM
    
    /* Stack at end of RAM */
    . = ALIGN(16);
    _stack_top = . + 0x10000; /* 64KB stack */
}