# NES Emulator

## State of the implementation

### Working

- CPU
- PPU (basic support)
- Joypad

### TODO

- APU (+ IRQ)
- Make advanced PPU 
- Support for iNES 2 ROM
- Make a basic interface (Load ROM, reset emulator, ...)
- Add support for external controllers (Wired or using Bluetooth)

## Game tested (NTSC Only)

- Pac-Man (OK)
- Super Mario Bros (Time and score are scrolling with the rest of the game) 
- Zelda (Not working)

## Resources Used

### Global
- [Writing NES Emulator in Rust](https://bugzmanov.github.io/nes_ebook/chapter_1.html) 
- [The Rust Programming Language](https://doc.rust-lang.org/book/)

### CPU 
- [Easy 6502](https://skilldrick.github.io/easy6502/)
- [6502 Reference](https://www.nesdev.org/obelisk-6502-guide/reference.html)
- [An introduction to 6502 math: addition, subtraction and more](https://retro64.altervista.org/blog/an-introduction-to-6502-math-addiction-subtraction-and-more/)
- [6502 Algorithms](https://cx16.dk/6502/algorithms.html)
- [Unintended Opcodes](https://hitmen.c02.at/files/docs/c64/NoMoreSecrets-NMOS6510UnintendedOpcodes-20162412.pdf)

### PPU
- [Nintendo Entertainment System Architecture](https://fms.komkon.org/EMUL8/NES.html#LABH)

### APU
- [NESDev APU](https://www.nesdev.org/wiki/APU)
- [NESDev APU Envelope](https://www.nesdev.org/wiki/APU_Envelope) 
- [NESDev APU Mixer](https://www.nesdev.org/wiki/APU_Mixer)