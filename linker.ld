ENTRY(_start)

SECTIONS {
    . = 0x7c00;

    .boot : { *(.boot) }
    .text : { *(.text) }
    .rodata : { *(.rodata) }
    .data :
    {
        *(.rodata)
        *(.data)
    }

    . = 0x7c00 + 510;

    .magic_number :
    {
        SHORT(0xaa55)       /* magic number for bootable disk */
    }
}