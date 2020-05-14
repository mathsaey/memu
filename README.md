# memu

Emulators written in Rust and reusable abstractions for their implementations.

This is a hobby project where I explore writing simple emulators in rust. The goal is twofold:

- Become familiar with Rust
- Dip my toes in Emulator development

To do this, I decided to build emulators for various devices, starting with a [CHIP 8](https://en.wikipedia.org/wiki/CHIP-8) emulator.
Ideally, some of the abstractions created for this purpose can be reused for a future emulator.

Since I am using this code mainly as a way to mess around with Rust do not expect idiomatic code here.
Do expect a lot of overengineering as I figure out what I can and cannot do with Rust.

Details about the various emulators can be found in `src/<name of emulator>`.

## Current Status

Currently, memu can emulate the chip8 system, albeit without sound.

## Installation and Use

Use `cargo` to fetch dependencies and build the application:

- `cargo build`

Afterwards, run the generated binary, pass the `--help` switch to find out the available options:

- `cargo run -- --help`

For more serious use, install the generated binary somewhere, so you don't always need to use `cargo run -- …` to run the application.
If you want to actually use this to play games, be sure to build in `release`: `cargo build --release`.

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

