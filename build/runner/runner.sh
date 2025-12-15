#!/bin/sh

qemu-system-x86_64 \
    -M q35 \
    -drive id=disk0,format=raw,file=build/image.bin,if=none \
    -device ide-hd,drive=disk0,bus=ide.0 \
    -monitor stdio
