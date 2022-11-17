//! Contains structs related to input programs that can be run on the system. The `Program` struct is responsible for the representation of bytes contained in a given
//! input. The `Instruction` struct contains the four hexadecimal digits that represent a single instruction, and the functionality to run it on a given system state.

use rand::Rng;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use crate::system;
use crate::utils::{big_endian_4_2, big_endian_4_3};

#[derive(Copy, Clone)]
/// Represents the instructions of a program's byte code as four hexadecimal digits (unsigned 4-bit integers). 
/// 
/// Instructions can be parsed from a single 16-bit unsigned integer, and, given a system state, run to update the state.
/// 
/// 
pub struct Instruction(u8, u8, u8, u8);

impl From<u16> for Instruction {

    /// Parses an Instruction from a single 16-bit unsigned integer.
    /// 
    /// # Example
    /// ```
    /// let instruction: Instruction = 0xD01Fu16.into();
    /// assert_eq!(instruction.3, 0xF);
    /// ```
    fn from(value: u16) -> Self {
        let b1 = (value & 0b1111000000000000) >> 12;
        let b2 = (value & 0b0000111100000000) >> 8;
        let b3 = (value & 0b0000000011110000) >> 4;
        let b4 = value & 0b0000000000001111;
        Instruction(b1 as u8, b2 as u8, b3 as u8, b4 as u8)
    }
}

impl Instruction {

    /// Decodes and executes an Instruction given the mutable state of a `System`.
    /// 
    /// The exact action taken by this method depends on the instruction itself. Typically, the first digit represents the action to be made,
    /// and the remaining digits contain additional information, such as parameters, for the execution.
    /// 
    /// # Panics
    /// If an instruction is passed that cannot be decoded, a panic is raised.
    /// 
    pub fn execute(self, sys: &mut system::System) {
        match self {
            Instruction(0, 0, 0xE, 0) => { //DISPLAY Clear
                sys.memory.clear_display();
            },
            Instruction(0, 0, 0xE, 0xE) => { //RETURN
                sys.pc = sys.stack.pop().unwrap();
            },
            Instruction(1, n1, n2, n3) => { //JUMP
                let address = big_endian_4_3(n1, n2, n3);
                sys.pc = address;
            },
            Instruction(2, n1, n2, n3) => { //CALL
                let address = big_endian_4_3(n1, n2, n3);
                sys.stack.push(sys.pc);
                sys.pc = address;
            },
            Instruction(0, n1, n2, n3) => { //CALL MACHINE
                let _address = big_endian_4_3(n1, n2, n3);
                //SKIP
            },
            Instruction(3, x, n1, n2) => { //Skip if VX == NN
                let val = big_endian_4_2(n1, n2);
                let v_val = sys.registers.get(x);
                if val == v_val {
                    sys.increment_pc();
                }
            },
            Instruction(4, x, n1, n2) => { //Skip if VX != NN
                let val = big_endian_4_2(n1, n2);
                let v_val = sys.registers.get(x);
                if val != v_val {
                    sys.increment_pc();
                }
            },
            Instruction(5, x, y, 0) => { //Skip if VX == VY
                let vx_val = sys.registers.get(x);
                let vy_val = sys.registers.get(y);
                if vx_val == vy_val {
                    sys.increment_pc();
                }
            },
            Instruction(6, x, n1, n2) => { //VX = NN
                let val = big_endian_4_2(n1, n2);
                sys.registers.set(x, val);
            },
            Instruction(7, x, n1, n2) => { //VX += NN (no carry)
                let val = big_endian_4_2(n1, n2);
                sys.registers.set(x, (val as u16 + sys.registers.get(x)as u16) as u8);
            },
            Instruction(8, x, y, 0) => { //VX = VY
                sys.registers.set(x, sys.registers.get(y));
            },
            Instruction(8, x, y, 1) => { //VX |= VY
                sys.registers.set(x, sys.registers.get(x) | sys.registers.get(y));
            },
            Instruction(8, x, y, 2) => { //VX &= VY
                sys.registers.set(x, sys.registers.get(x) & sys.registers.get(y));
            },
            Instruction(8, x, y, 3) => { //VX ^= VY
                sys.registers.set(x, sys.registers.get(x) ^ sys.registers.get(y));
            },
            Instruction(8, x, y, 4) => { //VX += VY (may set VF carry flag)
                let mut sum = sys.registers.get(x) as u16 + sys.registers.get(y) as u16;
                sys.registers.set_vF(0);
                if sum >= 0x100 {
                    sum -= 0x100;
                    sys.registers.set_vF(1);
                }
                sys.registers.set(x, sum as u8);
            },
            Instruction(8, x, y, 5) => { //VX -= VY (may un-set VF carry flag on borrow)
                let mut sum = 0x100 + sys.registers.get(x) as u16 - sys.registers.get(y) as u16;
                sys.registers.set_vF(0);
                if sum >= 0x100 {
                    sum -= 0x100;
                    sys.registers.set_vF(1);
                }
                sys.registers.set(x, sum as u8);
            },
            Instruction(8, x, _, 6) => { //VX shifted right by 1, lsb set to VF
                let val = sys.registers.get(x);
                sys.registers.set_vF(x & 1);
                sys.registers.set(x, val >> 1);
            },
            Instruction(8, x, y, 7) => { //VX = VY - VX (may un-set VF carry flag on borrow)
                let mut sum = 0x100 + sys.registers.get(y) as u16 - sys.registers.get(x) as u16;
                sys.registers.set_vF(0);
                if sum >= 0x100 {
                    sum -= 0x100;
                    sys.registers.set_vF(1);
                }
                sys.registers.set(x, sum as u8);
            },
            Instruction(8, x, _, 0xE) => { //VX shifted left by 1, msb set to VF
                let mut val = sys.registers.get(x) as u16;
                sys.registers.set_vF((x & 0b10000000) >> 7);
                val <<= 1;
                if val > 0x100 {
                    val -= 0x100;
                }
                sys.registers.set(x, val as u8);
            },
            Instruction(9, x, y, 0) => { //Skip if VX != VY
                let vx_val = sys.registers.get(x);
                let vy_val = sys.registers.get(y);
                if vx_val != vy_val {
                    sys.increment_pc();
                }
            },
            Instruction(0xA, n1, n2, n3) => { //I = NNN
                let address = big_endian_4_3(n1, n2, n3);
                sys.registers.set_i(address);
            },
            Instruction(0xB, n1, n2, n3) => { //Jump to NNN + V0
                let address = big_endian_4_3(n1, n2, n3);
                let v0_val = sys.registers.get(0);
                sys.pc = address + v0_val as u16;
            },
            Instruction(0xC, x, n1, n2) => { //VX = rand(0-255) & NN
                let val = big_endian_4_2(n1, n2);
                let r = sys.rng.gen_range(0..=255u8) & val;
                sys.registers.set(x, r);
            },
            Instruction(0xD, x, y, n) => { //draw(sprite(x: VX, y: VY, w: 8, h: N)), sprite defined at I, VF set if anything is drawn
                let x_pos = sys.registers.get(x) % 64;
                let y_pos = sys.registers.get(y) % 32;
                sys.registers.set_vF(0);

                for i in 0..n {

                    if y_pos + i >= sys.screen_height {
                        break;
                    }

                    let sprite_byte = sys.memory.get(sys.registers.i() + i as u16);
                    for j in 0..8u8 {

                        if x_pos + j >= sys.screen_width {
                            break;
                        }

                        if sprite_byte & (1 << (7 - j)) == 0 {
                            continue;
                        }
                        if sys.memory.flip_pixel(x_pos + j, y_pos + i) {
                            sys.registers.set_vF(1);
                        }
                    }
                }
            },
            Instruction(0xE, x, 0x9, 0xE) => { //Skip if key x is pressed
                if sys.keyboard.get(x) {
                    sys.increment_pc();
                }
            },
            Instruction(0xE, x, 0xA, 0x1) => { //Skip if key x is not pressed
                if !sys.keyboard.get(x) {
                    sys.increment_pc();
                }
            },
            Instruction(0xF, x, 0x0, 0x7) => { //VX = delay timer
                sys.registers.set(x, sys.delay_timer.get());
            },
            Instruction(0xF, x, 0x0, 0xA) => { //VX = await key()
                let l = sys.keyboard.latest();
                if l == 16 {
                    sys.pc -= 2;
                }
                else {
                    sys.registers.set(x, l);
                }
            },
            Instruction(0xF, x, 0x1, 0x5) => { //delay timer = VX
                sys.delay_timer.set(sys.registers.get(x));
            },
            Instruction(0xF, x, 0x1, 0x8) => { //sound timer = VX
                sys.sound_timer.set(sys.registers.get(x));
            },
            Instruction(0xF, x, 0x1, 0xE) => { //I += VX
                let mut val = sys.registers.i() + sys.registers.get(x) as u16;
                if val >= 0x1000 {
                    val -= 0x1000;
                    sys.registers.set_vF(1);
                }

                sys.registers.set_i(val);
            },
            Instruction(0xF, x, 0x2, 0x9) => { //I = address of sprite VX
                let c = sys.registers.get(x) & 0xF;
                sys.registers.set_i(0x50u16 + 5u16 * c as u16);
            },
            Instruction(0xF, x, 0x3, 0x3) => { //Convert VX to decimal. Store 100-digit at *I, 10-digit at *(I+1) and 1-digit at *(I+2).
                let value = sys.registers.get(x);
                sys.memory.store(sys.registers.i(), value / 100);
                sys.memory.store(sys.registers.i() + 1, (value % 100) / 10);
                sys.memory.store(sys.registers.i() + 2, value % 10);
            },
            Instruction(0xF, x, 0x5, 0x5) => { //Store [V0..VX] in memory at [*I, *(I+1),...]
                for i in 0..=x {
                    sys.memory.store(sys.registers.i() + i as u16, sys.registers.get(i));
                }
            },
            Instruction(0xF, x, 0x6, 0x5) => { //Loads [V0..VX] from memory at [*I, *(I+1),...]
                for i in 0..=x {
                    sys.registers.set(i, sys.memory.get(sys.registers.i() + i as u16));
                }
            },

            _ => panic!(),
        }
    }
}

impl std::fmt::Display for Instruction {
    
    /// Formats the `Instruction` struct as `INSTR: XXXX` where each `X` represents a hexadecimal digit.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "INSTR: {:X} {:X} {:X} {:X}", self.0, self.1, self.2, self.3)
    }
}

/// Represents a program's bytecode as a list of bytes
pub struct Program {
    pub instructions: Vec<u8>,
}

impl Program {

    /// Attempts to load a program from a given file path
    /// 
    /// # Example
    /// ```
    /// let program = Program::load("rom.ch8")?;
    /// ```
    pub fn load<P>(path: P) -> io::Result<Program> 
        where P: AsRef<Path>, {
            let file = File::open(path)?;
            Ok(Program { instructions: file.bytes().filter_map(|b| b.ok()).collect() })
    }
}

impl std::fmt::Display for Program {

    /// Formats the `Program` struct as `<line>: <instruction>` where `line` and `instruction` are both represented as hexadecimal numbers.  
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.instructions.len()/2 {
            let x = 0x100u16 * *self.instructions.get(2*i).unwrap() as u16 + *self.instructions.get(2*i + 1).unwrap() as u16;
            writeln!(f, "{:0>2X}: {:0>4X}", i, x)?;
        }
        write!(f, "")
    }
}