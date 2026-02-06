@echo off


qemu-system-x86_64 ^
    -M q35 ^
    -drive id=disk0,file=build/image.bin,if=none,format=raw ^
    -device ide-hd,drive=disk0,bus=ide.0,rotation_rate=1 ^
    -monitor stdio
