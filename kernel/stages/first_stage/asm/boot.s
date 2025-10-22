// ANCHOR: segment
//; This will define a boot section for this asm code,
//; which we can put at the start of our binary.
.section .boot, "awx"
.global start
.code16
start:
    //; Disable interrupts
    cli
    //; zero segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
// ANCHOR_END: segment

// ANCHOR: stack
    //; clear the direction flag (e.g. go forward in memory when using
    //; instructions like lodsb)
    cld

    //; initialize stack
    mov sp, 0x7c00
// ANCHOR_END: stack

// ANCHOR: A20
//; Enable the A20 line via I/O Port 0x92
//; This method might not work on all motherboards
//; Use with care on real hardware!
enable_a20:
    //; Check if a20 is already enabled
    in al, 0x92
    test al, 2

    //; If so, skip the enabling code
    jnz enable_a20_after

    //; Else, enable the a20 line
    or al, 2
    and al, 0xFE
    out 0x92, al
enable_a20_after:
// ANCHOR_END: A20

// ANCHOR: INT13
check_int13h_extensions:

    //; Set function constants `dl` already contains the driver
    mov ah, 0x41
    mov bx, 0x55aa
    int 0x13
    jnc .int13_pass
    //; hlt system if there is no support
    hlt
.int13_pass:
// ANCHOR_END: INT13

// ANCHOR: disk
//; push disk number into the stack will be at 0x7bfe and call the first_stage function
push dx
call first_stage
// ANCHOR_END: disk
