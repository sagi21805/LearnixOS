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

    . = 0x7c00 + 446;

    /* Save the place for the partition table (rust will have a _partition table var) */
    _partition_table = .;
    .partition_table :
    {
        /* partition table entry 0 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 1 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 2 */
        QUAD(0)
        QUAD(0)
        /* partition table entry 3 */
        QUAD(0)
        QUAD(0)
    }
    . = 0x7c00 + 510;

    .magic_number :
    {
        SHORT(0xaa55)       /* magic number for bootable disk */
    }

}