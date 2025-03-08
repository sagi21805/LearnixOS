ENTRY(.first_stage_entry)

SECTIONS {

    .first_stage : { *(.first_stage) }
    
    . = 446;

    /* Make sure code doesn't override partition table */
    .partition_table : {
        QUAD(0)  QUAD(0)  /* Entry 0 */
        QUAD(0)  QUAD(0)  /* Entry 1 */
        QUAD(0)  QUAD(0)  /* Entry 2 */
        QUAD(0)  QUAD(0)  /* Entry 3 */
    }
    /* Make sector bootable */
    .magic_number : { SHORT(0xaa55) }

    . = 512;
  
}