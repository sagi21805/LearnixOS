ENTRY(.second_stage_entry)

SECTIONS {

    . = 512;

    .second_stage_entry : {
        *(.third_stage_entry)
    }

    .second_stage : {
        *(.third_stage)
    }

}