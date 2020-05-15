# memu

Emulators written in Rust and reusable abstractions for their implementations.

This is a hobby project where I explore writing simple emulators in rust. The goal is twofold:

- Become familiar with Rust
- Dip my toes in Emulator development

To do this, I decided to build emulators for various devices, starting with a [CHIP 8](https://en.wikipedia.org/wiki/CHIP-8) emulator.
Ideally, some of the abstractions created for this purpose can be reused for a future emulator.

Since I am using this code mainly as a way to mess around with Rust do not expect idiomatic code here.
Do expect a lot of overengineering as I figure out what I can and cannot do with Rust.

Details about a given emulator can be found in `src/<name of emulator>`.

## Current Status

Currently, memu can emulate the chip8 system, albeit without sound.
Not that many roms were tested, so expect some bugs.

## Build / Installation

Fetch the code, and use `cargo` to build and run it:

- `git clone https://github.com/mathsaey/memu.git`
- `cd memu`
- `cargo run`

If you're so inclined, you can use `cargo install --path .` to install memu.

## Use

To use memu, call it with an emulator name and a path to a rom for that emulator: `memu <emulator> <rom-path>`.
To use your terminal as a debug view, pass the `-D` flag; for a full list of options, use `memu --help`.
If you are using `cargo run`, replace `memu` with `cargo run --`.

Once the emulator is running, use `<esc>` to close it.

### Emulation modes

memu supports 3 different emulation modes:

- _cycle_: In cycle mode, the emulator executes a single instruction every time `<space>` is pressed.
  The emulator will start in cycle mode if the debug view is enabled (`-D`).
  Pressing `/` will change the emulator to _frame_ mode.
- _frame_: In frame mode, pressing `<space>` will make the emulator execute instructions until a redraw is required.
  Pressing `/` will put you in normal mode.
- _normal_: In this mode, the emulator runs at its normal speed.
  Pressing `<` or `>` will slow down or speed up the emulation speed, respectively.
  Press `/` to change to _cycle_ mode.
  Emulation starts in this mode if the debug view is not enabled.

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

