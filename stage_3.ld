ENTRY(start)

SECTIONS {

    . = 0x00100000;

    .start : {
        *(.start)
    }

    .kernel_function : {
        *(.kernel_function)
    }

}