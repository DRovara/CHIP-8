//! A simple implementation of a CHIP-8 emulator
//! 
//! I mainly used this to get started with rust. Rendering is performed in the terminal. Sound output is not currently supported.
//! The fetch/decode/execute loop supports arbitrary execution speed, however, with the time requirements of printing to stdout,
//! there is a hard cap on the maximum reachable speed.
//! 
//! Please make sure that your terminal can show at least 34 rows at once to run the emulator, otherwise weird graphic glitches will occur.

mod utils;
mod system;
mod program;

use std::io;
#[deny(missing_docs)]
/// Runs the emulator. The program to be run is hardcoded in the `main` function. You can change it by pasting your program of choice in the `test/data`
/// directory and then changing the value of the `name` variable accordingly. 
fn main() {   
    let stdin = io::stdin();

    let mut sys = system::System::new();
    let mut display = system::Display::new();

    let name = "tombstontipp";
    let program = program::Program::load("test/data/".to_string() + name + ".ch8").unwrap();

    println!("Program:\n{}", program);
    let mut string = String::new();
    let _res = stdin.read_line(&mut string);
    
    sys.load(program);
    print!("{}[2J", 27 as char);
    sys.run(&mut display);
}
