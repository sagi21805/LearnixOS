#!/bin/sh

qemu-system-x86_64 \
    -drive id=disk,format=raw,file=build/image.bin,if=none \
    -device ahci,id=ahci \
    -device ide-hd,drive=disk,bus=ahci.0 \
    -monitor stdio