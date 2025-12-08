@echo off


qemu-system-x86_64 ^
    -M q35 ^
    -drive id=disk0,file=build/image.bin,if=none,format=raw ^
    -device ahci,id=ahci0 ^
    -device ide-hd,drive=disk0,bus=ahci0.0 ^
    -monitor stdio