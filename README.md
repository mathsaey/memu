# memu

Emulators written in Rust and reusable abstractions for their implementations.

This is a hobby project where I explore writing simple emulators in rust. The goal is twofold:

- Become familiar with Rust
- Dip my toes in Emulator development

To do this, I decided to build emulators for various devices, starting with a [CHIP 8](https://en.wikipedia.org/wiki/CHIP-8) emulator.
Ideally, some of the abstractions created for this purpose can be reused for a future emulator.

Since I am using this code mainly as a way to mess around with Rust do not expect idiomatic code here.
Do expect a lot of overengineering as I figure out what I can and cannot do with Rust.

## Installation and Use

Simply build and install with cargo:

- `cargo build`
- `cargo run`

If you actually want to use this to play games, use the `--release` flag when building memu.

# (Non) Goals

As this is a hobby project, I'm using this project as an excuse to mess with various things such as:

- Emulation
  - Simple, reasonably efficient chip 8 emulator
- Non-Functional goals
  - Reusable components to use for additional emulators
  - A TUI interface that shows the internals of the system that is being emulated.

For the same reason I am not focusing on the following requirements:

- Perfect emulation
- Hyper-optimized code

# Resources

I model the chip-8 instruction set based on the following resources:

- References
  - http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
  - https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference
- Roms
  - https://github.com/dmatlack/chip8/tree/master/roms
