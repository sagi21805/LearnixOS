ENTRY(_start)

SECTIONS {

    . = 0x7c00;

    .boot : {
        *(.boot) 
    }

    .screen_minimal : {
        *(.screen_minimal)
    }

    .disk_minimal : {
        *(.disk_minimal)
    }

    . = 0x7c00 + 446;

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

    .magic_number : {
        SHORT(0xaa55)       /* magic number for bootable disk */
    }
    
    . = 0x7c00 + 512;

    .second_stage : {
        *(.second_stage) 
    }

    .disk : {
        *(.disk)
    }

    .global_descriptor_table : {
        *(.global_descriptor_table)
    }

    /* Pad the rest of the space with zeroes */
    .zero_fill : {
        FILL(0);  /* Fill remaining bytes with zero */
    }

    . = 0x7c00 + 4096;
    
  
}