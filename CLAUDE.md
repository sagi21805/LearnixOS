# LearnixOS

LearnixOS is a hobby operating system written from scratch in Rust. I build it to learn how operating systems work. Treat every task with that in mind:

- Explain the why behind a change, not only the what. If a fix depends on some hardware or OS concept (paging, interrupts, atomics, allocator invariants), say which one and how it applies.
- I'd rather understand a smaller change than receive a large one I can't follow. Keep diffs focused, and ask before restructuring code I didn't ask you to touch.
- When I ask how to do something, point me in the right direction first. Write the full implementation only when I ask for it.
- If a hand-rolled solution would teach more than a crate dependency, say so before reaching for the crate.

## Skills

- **graphify**: for questions about the codebase's architecture, how crates relate, or where something lives. If `graphify-out/` exists, query the graph before grepping around. After large structural changes, suggest running `/graphify . --update`.
- **rust-style**: load it before writing, editing or reviewing any Rust code in this repo, including one-line fixes. Its no_std/kernel and proc-macro sections apply here.
- **unslop**: apply it to all prose you write: replies, doc comments, commit messages, PR descriptions, and book text. Never apply it to code, compiler output or anything quoted verbatim.

## Documentation: the LearnixOS book

The book that explains this kernel chapter by chapter lives in a separate repository, `../LearnixOS-book` (mdBook, chapters in `src/`), and is published at https://www.learnix-os.com/. Each chapter covers one subsystem: bootloader, synchronization, memory management, interrupts and so on.

- When working on a subsystem, read the matching chapter for the design intent and the terms I use.
- If code changes make a chapter inaccurate, tell me which chapter and section. Don't edit the book unless I ask.

## Layout

- `bootloader/first_stage`: 16-bit real-mode stage
- `bootloader/second_stage`: 32-bit protected-mode stage
- `kernel/`: the 64-bit kernel
- `crates/`: kernel libraries, all `no_std`
  - `arch/x86`: architecture-specific code
  - `memory/{bump,buddy,page,slab}`: allocators
  - `sync`: synchronization primitives
  - `drivers/`: keyboard, VGA display (`ahci` and `pci` are currently disabled in the workspace)
  - `macros`: proc macros
  - `common`, `libk`: shared kernel utilities
- `xtask/`: build tooling

## Building and running

The project uses nightly Rust (see `rust-toolchain.toml`) and runs under QEMU.

```bash
cargo xtask build              # build all stages into image.bin
cargo xtask run                # build and boot in QEMU
cargo xtask run --release
cargo xtask run --release --debug-package <crate>   # keep debug assertions for one crate
```

Each stage is built against its own JSON target spec with `-Z build-std`, so a plain `cargo build` at the workspace root won't produce a bootable image. Use `xtask`.
