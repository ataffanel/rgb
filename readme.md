# rgb: a Rust GameBoy emulator

This repos contains a small toy GameBoy emulator written in Rust.

This project was created in ~2016 mainly as a way to learn Rust and to play with
emulation. It does not aim at being the must accurate or fast GameBoy emulator
but more as being a Rust playground that I can dig into from time to time.

## Status

The CPU is almost fully accurate, input works and the video is good enough for
most games. There has been no attempt to implement sound so far.

Supports reading and writing `.sav` file for saving game progress. A minimal
bootstrap ROM is included, but another one can be provided on the command line.

Only the original DMG gameboy is implemented.

## Running

Assumming you have rust and cargo installed, can be run with:
```
cargo run -- <rom>
```

`--help` will print a list of supported command line arguments.

## Architecture

The emulator is separated in two Rust crates: `rgb-core` and `rgb-sdl`. `rgb-core` implements DMG emulation, `rgb-sdl` implenent graphical output and gamepad input using SDL. The
intent is to make the core portable to more than running in a window. Currently
`rgb-core` only depends on `std`, it does not uses threads and has no other
dependencies, this makes it very portable.

There has already been successful experiment of running the emulator in a web
browser and as a libretro (retroarch) core. Future experiment might include
trying to run in an embedded system, this would require to remove the `std`
dependency from `rgb-core` though.

