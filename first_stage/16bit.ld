ENTRY(_start)

SECTIONS {

    . = 0x7c00;

    .boot : { *(.boot) }

    .start : { *(.start) }

    .first_stage : { *(.first_stage)}
    
    . = 0x7c00 + 446;

    /* Make sure code doesn't override partition table */
    .partition_table : {
        QUAD(0)  QUAD(0)  /* Entry 0 */
        QUAD(0)  QUAD(0)  /* Entry 1 */
        QUAD(0)  QUAD(0)  /* Entry 2 */
        QUAD(0)  QUAD(0)  /* Entry 3 */
    }
    /* Make sector bootable */
    .magic_number : { SHORT(0xaa55) }

    . = 0x7c00 +512;
  
}