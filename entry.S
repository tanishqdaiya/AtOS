.section ".text.boot"
.global _start

_start:
    // set stack pointer
    ldr x0, =_stack_top
    mov sp, x0

    // jump to rust
    bl _rust_main

1:
    wfe
    b 1b
