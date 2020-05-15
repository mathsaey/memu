# memu Chip 8

[CHIP 8](https://en.wikipedia.org/wiki/CHIP-8) emulation.

# Controls

The layout of the CHIP8 hex keypad:

| key | key | key | key |
|---|---|---|---|
| 1 | 2 | 3 | C |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

is mapped onto a qwerty keyboard:

| key | key | key | key |
|---|---|---|---|
| 1 | 2 | 3 | 4 |
| q | w | e | r |
| a | s | d | f |
| z | x | c | v |

# Notes

- The operation names in the log output and debug views are based on the [cowgod reference](devernay.free.fr/hacks/chip8/C8TECH10.HTM).

# Resources

I modelled the chip-8 instruction set based on the following resources:

- References
  - http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
  - http://mattmik.com/files/chip8/mastering/chip8.html
  - https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference
- Roms
  - https://github.com/dmatlack/chip8/tree/master/roms
