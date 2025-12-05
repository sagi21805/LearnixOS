@echo off


qemu-system-x86_64 ^
    -drive id=Disk,file=build/image.bin,if=none,format=raw ^
    -device ahci,id=ahci0 ^
    -device ide-hd,drive=Disk,bus=ahci0.0 
    -monitor stdio