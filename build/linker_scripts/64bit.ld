ENTRY(_start)

SECTIONS {
    . = 0x9000;

    .start : { *(.start) }

    .text : { *(.text .text.*) }
    .bss : { *(.bss .bss.*) }
    .rodata : { *(.rodata .rodata.*) }
    .data : { *(.data .data.*) }
    .eh_frame : { *(.eh_frame .eh_frame.*) }
    .eh_frame_hdr : { *(.eh_frame_hdr .eh_frame_hdr.*) }
}
