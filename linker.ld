ENTRY(_start)

SECTIONS
{
    . = 0x80000;

    .text :
    {
        *(.text.boot)
        *(.text.vectors)
        *(.text*)
    }

    .rodata : { *(.rodata*) }
    .data   : { *(.data*) }
    .bss    : { *(.bss*) }

    _kernel_top = .;

    . = ALIGN(16);
    _stack_top = . + 0x4000;
}
