ENTRY(_start)

SECTIONS
{
    . = 0x80000;

    .text :
    {
        *(.text.boot)
        *(.text*)
    }

    .rodata : { *(.rodata*) }
    .data   : { *(.data*) }
    .bss    : { *(.bss*) }

    . = ALIGN(16);
    _stack_top = . + 0x4000;
}
