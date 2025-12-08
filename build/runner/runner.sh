#!/bin/sh

qemu-system-x86_64 \
    -M q35 \
    -drive id=disk0,format=raw,file=build/image.bin,if=none \
    -device ahci,id=ahci0 \
    -device ide-hd,drive=disk0,bus=ahci0.0 \
    -monitor stdio