#!/bin/sh

qemu-system-x86_64 \
    -M q35 \
    -drive id=disk0,format=raw,file=build/image.bin,if=none \
    -drive id=disk1,format=raw,file=build/bin/first_stage,if=none \
    -device ide-hd,drive=disk0,bus=ide.0 \
    -device ide-hd,drive=disk1,bus=ide.1 \
    -monitor stdio
