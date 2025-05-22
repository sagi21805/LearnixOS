ENTRY(_start)

SECTIONS {
    . = 0x7c00 + 512;

    .page_table : {
        *(.page_table)
    }

    .start : { *(.start) }

    .text : { *(.text .text.*) }
    .bss : { *(.bss .bss.*) }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .eh_frame : { *(.eh_frame .eh_frame.*) }
    .eh_frame_hdr : { *(.eh_frame_hdr .eh_frame_hdr.*) }

    .fill : {
        FILL(0)
        . = 0xe000;
    }
}
