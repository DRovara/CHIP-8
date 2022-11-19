# CHIP-8 Emulator

This is a very simplistic CHIP-8 emulator, that I implemented as an exercise to get acquainted with the Rust programming language.
It renders the display contents in the Terminal. As such, it is not very well-optimized, but it does support all basic CHIP-8 features and you can use it to run games. Checking key-input uses `winapi` so it only supports windows.

***Run using:*** `cargo run`

### Future changes

Here is a list of possible changes that could still be made to improve this project:

- Only render pixels that actually changed on each display update.