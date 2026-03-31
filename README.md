<p align="center">
  <a href="https://github.com/sponsors/sagi21805">
    <img src="https://img.shields.io/badge/Sponsor-me-ff69b4?style=for-the-badge&logo=github-sponsors" alt="Sponsor">
  </a>
</p>

# The Learnix Opearting System

This repository contains the source code of the LearnixOS kernel — an educational operating system written in Rust.

This operating system is built from scratch and explained in detail in the [LearnixOS Book](https://www.learnix-os.com/) which aims to explain the subject of operating system while implementing one.

# Running the project

This projects uses QEMU as the virtualization layer to run the kernel locally.

To download QEMU for your platform, click [here](https://www.qemu.org/download/)

This project is built in Rust as the main programming language.

To download Rust for your platform click [here](https://rust-lang.org/learn/get-started/)

Then simply build the project with 

```bash
cargo xtask run
```

## License

This codebase is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

You can freely use, modify, and redistribute this code under either license, at your option.
