.section ".text.boot"
.global _start

_start:
    // set stack pointer
    ldr     x0, =_stack_top
    mov     sp, x0

    // load exception vector table for EL1
    adr     x0, el1_vectors
    msr     VBAR_EL1, x0
    isb

    // checking current exception level
    mrs     x0, CurrentEL
    cmp     x0, #0x8 // EL2?
    b.ne    el1_start
    
    // EL stack pointer
    ldr     x0, =_stack_top
    msr     SP_EL1, x0
    
    mov     x0, #(1 << 31) // bit 31 selects Aarch32/64
    msr     HCR_EL2, x0 // HCR_EL2 is like "a rulebook that EL2 writes for EL1"

    // set SPSR_EL2 to EL1h, basically once we ERET, we must switch to EL1h mode
    mov     x0, #(0b0101)  // EL1h
    orr     x0, x0, #(0b1111 << 6)
    msr     SPSR_EL2, x0

    // finally set the ERET PC
    adr     x0, _rust_main
    msr     ELR_EL2, x0

    eret // insane

el1_start:
    bl _rust_main

1:
    wfe
    b 1b
