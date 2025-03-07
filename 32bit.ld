ENTRY(start)

SECTIONS {

    .second_stage_entry : {
        *(.third_stage_entry)
    }

    .second_stage : {
        *(.third_stage)
    }

}