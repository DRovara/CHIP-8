//! A collection of structs and functions used to represent the state of a CHIP-8 system.

extern crate user32;
use rand::rngs::ThreadRng as ThreadRng;
use std::{thread};
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use crate::program::{self, Instruction};
use crate::utils::{big_endian_8_2};

#[deny(missing_docs)]

/// Represents the main memory of a CHIP-8 system. In our implementation, it contains 4096 bytes that can be accessed and modified using the `get(...)` and `store(...)` methods.
/// 
/// Also provides functionality for the access of the display buffer, which is stored in the last 0x100 bytes of the memory.
pub struct Memory {
    memory: [u8; 4096],
}

impl Memory {

    /// Creates a new `Memory` object.
    /// 
    /// Font data for the sprites of all 16 hexadecimal digits is immediately loaded into the address space 0x50-0x9F.
    /// 
    /// # Example
    /// ```
    /// let mem = Memory::new();
    /// ```
    /// 
    pub fn new() -> Memory {
        let mut mem = Memory { memory: [0u8; 4096] };
        let font_sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            0x20, 0x60, 0x20, 0x20, 0x70,
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            0x90, 0x90, 0xF0, 0x10, 0x10,
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            0xF0, 0x10, 0x20, 0x40, 0x40,
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            0xF0, 0x80, 0xF0, 0x80, 0x80
        ];
        for (idx, byte) in font_sprites.into_iter().enumerate() {
            mem.store(0x50 + idx as u16, byte);
        }

        mem
    }

    /// Fetches the value of the byte at a given 12-bit address.
    /// 
    /// The address is represented as a `u16` in Rust, but the address space only has a size of 12 bits. Accessing a higher address will return `0`.
    /// 
    /// # Example
    /// ```
    /// let mem = Memory::new();
    /// let x = mem.get(0x50);
    /// ```
    /// 
    pub fn get(&self, address: u16) -> u8 {
        if address as usize >= self.memory.len() {
            return 0;
        }
        self.memory[address as usize]
    }

    /// Stores a given 8-bit value to a 12-bit address.
    /// 
    /// The address is represented as a `u16` in Rust, but the address space only has a size of 12 bits. Accessing a higher address will result in a panic.
    /// 
    /// # Example
    /// ```
    /// let mut mem = Memory::new();
    /// mem.store(0x300, 42);
    /// ```
    /// 
    pub fn store(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }

    /// Sets or resets the pixel at the given coordinates.
    /// 
    /// If the pixel was already set, it will be reset and `true` will be returned. Otherwise, it will be set and `false` is returned.
    /// 
    /// # Example
    /// ```
    /// let mut mem = Memory::new();
    /// let was_set = mem.flip_pixel(42, 24);
    /// ```
    /// 
    pub fn flip_pixel(&mut self, x: u8, y: u8) -> bool {
        let idx = 0xF00 + (x as u16 + y as u16 * 64u16) / 8;
        let value = 1 << (7 - x % 8);
        let current = self.get(idx);
        let reset = (current & value) > 0;
        self.store(idx, current ^ value);
        reset
    }

    /// Clears the display buffer
    /// 
    /// The display buffer occupies address space 0xF00-0xFFF. This method resets all bytes in this space to 0.
    /// 
    /// # Example
    /// ```
    /// let mut mem = Memory::new();
    /// mem.clear_display();
    /// ```
    /// 
    pub fn clear_display(&mut self) {
        for i in 0xF00..=0xFFF {
            self.store(i, 0);
        }
    }
}

impl std::fmt::Display for Memory {

    /// Formats the `Memory` struct as a table of width 32 and height 128, where each cell corresponds to the current value of the byte it represents in storage.  
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "     ")?;
        for i in 0..32 {
            write!(f, "{:0>2X} ", i)?;
        }
        writeln!(f)?;
        write!(f, "     ------------------------------------------------------------------------------------------------")?;
        writeln!(f)?;
        for i in 0..128 {
            write!(f, "{:0>3X}: ", i * 32)?;
            for j in 0..32 {
                write!(f, "{:0>2X} ", self.get(i*32+j))?;
            }
            writeln!(f)?;
        }
        write!(f, "")
    }
}

/// Represents the register array of the CHIP-8 processor.
/// 
/// A CHIP-8 processor consists of 16 `V` registers and one `I` register.
/// The `V`  registers each hold a single unsigned byte and are named `V0, V1, ..., VF` respectively. `VF` is generally used as a flag register to be set and
/// reset by specific instructions under certain conditions.
/// The `I` register holds a single 12-bit unsigned integer value that can be used to address the main memory. It is used for certain instructions to load information such
/// as sprite data from the main memory.
pub struct Registers {
    v: [u8; 16],
    i: u16,
}

impl Registers {

    /// Creates a new `Registers` object.
    /// 
    /// All 16 `V` registers, as well as the `I` register are initialized as `0`.
    /// 
    /// # Example
    /// ```
    /// let reg = Registers::new();
    /// ```
    /// 
    pub fn new() -> Registers {
        Registers { v: [0u8; 16], i: 0 }
    }

    /// Fetches the value of the byte in the `V` register addressed by `idx`.
    /// 
    /// The index is represented as a `u8` in Rust, but CHIP-8 only has a total of 16 `V` registers. Accessing a higher index will return 0.
    /// 
    /// # Example
    /// ```
    /// let reg = Registers::new();
    /// let x = reg.get(1);
    /// ```
    /// 
    pub fn get(&self, idx: u8) -> u8 {
        if idx as usize >= self.v.len() {
            return 0;
        }
        self.v[idx as usize]
    }

    /// Stores a given 8-bit value in the `V` register addressed by `idx`.
    /// 
    /// The index is represented as a `u8` in Rust, but CHIP-8 only has a total of 16 `V` registers. Accessing a higher index will result in a panic.
    /// 
    /// # Example
    /// ```
    /// let mut reg = Registers::new();
    /// reg.set(1, 42);
    /// ```
    /// 
    pub fn set(&mut self, idx: u8, val: u8) {
        self.v[idx as usize] = val;
    }

    /// Fetches the current value inside the `I` register.
    /// 
    /// # Example
    /// ```
    /// let reg = Registers::new();
    /// let x = reg.i(1);
    /// ```
    /// 
    pub fn i(&self) -> u16 {
        self.i
    }

    /// Stores a given 12-bit value in the `I` register.
    /// 
    /// The value is passed as a `u16`. However, the `I` register can only hol up to 12 bits, so values larger than 0xFFF should not be passed.
    /// 
    /// # Example
    /// ```
    /// let mut reg = Registers::new();
    /// reg.set_i(42);
    /// ```
    /// 
    pub fn set_i(&mut self, val: u16) {
        self.i = val;
    }

    #[allow(non_snake_case)]
    /// Sets the value of the `VF` flag register specifically.
    /// 
    /// # Example
    /// ```
    /// let mut reg = Registers::new();
    /// reg.set_vF(1);
    /// ```
    /// 
    pub fn set_vF(&mut self, value: u8) {
        self.v[15] = value;
    }

}

/// Represents the Stack used to store return addresses for `CALL` and `RETURN` instructions in the CHIP-8 instruction set. While the stack was typically located inside
/// the main memory on real CHIP-8 devices, we store it as a separate data structure with (practically) unlimited storage for our emulation.
pub struct Stack {
    stack: Vec<u16>,
}

impl Stack {

    /// Creates a new instance of the `Stack` struct.
    /// 
    /// The stack starts with size `0`.
    /// 
    /// # Example
    /// ```
    /// let stack = Stack::new();
    /// ```
    /// 
    pub fn new() -> Stack {
        Stack { stack: vec![] }
    }

    /// Pushes a value to the top of the stack.
    /// 
    /// # Example
    /// ```
    /// let mut stack = Stack::new();
    /// stack.push(42);
    /// ```
    /// 
    pub fn push(&mut self, val: u16) {
        self.stack.push(val);
    }

    /// Removes the topmost value from the stack and returns it as an `Option`.
    /// 
    /// If the stack was empty, returns `None` instead.
    /// 
    /// # Example
    /// ```
    /// let mut stack = Stack::new();
    /// stack.push(42);
    /// let fourty_two = stack.pop();
    /// ```
    /// 
    pub fn pop(&mut self) -> Option<u16> {
        self.stack.pop()
    }

}

/// Represents a timer in the CHIP-8 system. Timers can be set to 8-bit values and will then decrement at a rate of 60Hz until they reach `0`.
/// Typically, CHIP-8 has a `Delay Timer` and a `Sound Timer` with similar functionalities. Both of them can be represented
/// by a Timer struct.
pub struct Timer {
    value: u8,
    last_update: u128
}

impl Timer {

    /// Creates a new instance of the `Timer` struct, starting at value `0`.
    /// 
    /// # Example
    /// ```
    /// let timer = Timer::new();
    /// ```
    pub fn new() -> Timer {
        Timer { value: 0, last_update: 0 }
    }

    /// Ticks down the timer by `1` if it is larger than `0`.
    fn tick(&mut self) {
        if self.value > 0 {
            self.value -= 1;
        }
    }

    /// Sets the timer to a given 8-bit value.
    /// 
    /// After being set, the timer will decrement at a rate of 60 Hz until it reaches `0`.
    /// 
    /// # Example
    /// ```
    /// let mut timer = Timer::new();
    /// timer.set(42);
    /// ```
    pub fn set(&mut self, value: u8) {
        self.value = value
    }

    /// Gets the current value of the timer.
    /// 
    /// # Example
    /// ```
    /// let mut timer = Timer::new();
    /// timer.set(42);
    /// let fourty_two = timer.get();
    /// ```
    pub fn get(&self) -> u8 {
        self.value
    }

    /// Determines if the last call of the `tick()` method was longer than 16 ms ago. If so, it calls the `tick()` method to decrement the timer value.
    /// 
    /// # Example
    /// ```
    /// let mut timer = Timer::new();
    /// timer.set(42);
    /// loop {
    ///     if timer.get() == 0 {
    ///         break;
    ///     }
    ///     timer.update();
    /// }
    /// ```
    fn update(&mut self) {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let dt = current_time - self.last_update;
        if dt > 16 {
            self.tick();
            self.last_update = current_time;
        }
    }
}

/// Represents the state of the 16-key CHIP-8 keyboard.
pub struct Keyboad {
    keys: [bool; 16],
    latest: u8,
}

/// Maps 16 QWERTY keyboard keys to the corresponding CHIP-8 key index they should represent.
const KEYBOARD_KEYS: [u8; 16] = [
    b'X',
    b'1',
    b'2',
    b'3',
    b'Q',
    b'W',
    b'E',
    b'A',
    b'S',
    b'D',
    b'Z',
    b'C',
    b'4',
    b'R',
    b'F',
    b'V',
];


impl Keyboad {

    /// Creates a new instance of the `Keyboard` struct.
    /// 
    /// All keys are initialized as "not pressed".
    /// 
    /// # Example
    /// ```
    /// let kb = Keyboard::new();
    /// ```
    pub fn new() -> Keyboad {
        Keyboad { keys: [false; 16], latest: 16 }
    }

    /// Gets the current state of the key with the given index.
    /// 
    /// `true` indicates that the key is currently pressed, `false` indicates it is not pressed.
    /// 
    /// # Example
    /// ```
    /// let kb = Keyboard::new();
    /// let a_pressed = kb.get(0xA);
    /// ```
    pub fn get(&self, key: u8) -> bool {
        if key as usize >= self.keys.len() {
            return false;
        }
        self.keys[key as usize]
    }

    /// Updates the state of the keyboard.
    /// 
    /// Invokes the `user32::GetAsyncKeyState(...)` function for each possible key to get its current state and stores it. It also updates the value of the
    /// `latest` field, indicating the latest key that was pressed (or 0x10 if no key was pressed).
    pub fn update(&mut self) {
        self.latest = 16;
        for (idx, key) in KEYBOARD_KEYS.iter().enumerate() {
            if unsafe { user32::GetAsyncKeyState(*key as u8 as i32) } == -32767 {
                self.latest = idx as u8;
                self.keys[idx] = true;
            }
            else {
                self.keys[idx] = false;
            }
        }
    }

    /// Gets the index of the latest key that was pressed (or 0x10 if no key was pressed).
    /// 
    /// # Example
    /// ```
    /// let kb = Keyboard::new();
    /// let latest = kb.latest();
    /// ```
    pub fn latest(&self) -> u8 {
        self.latest
    }
}

/// A simulated `Display` for the CHIP-8, using stdout to draw the pixels. 
pub struct Display {
    pixels: [[u8;64]; 32],
}

impl Display {

    /// Creates a new instance of the `Display` struct, initializing its 32x64 pixel matrix as `OFF`.
    /// 
    /// # Example
    /// ```
    /// let display = Display::new();
    /// ```
    pub fn new() -> Display {
        Display { pixels: [[0u8;64]; 32] }
    }

    /// Updates the current state of the display by using the `Memory` component of the current `System` state.
    /// 
    /// If a pixel is set in the `memory`, its value will be set to `4` in the `pixels` matrix. If it was not set, its value will be decremented by `1` instead.
    /// Pixels are rendered in the console, as long as their value is larger than `0`.
    /// 
    /// # Example
    /// ```
    /// let mut system = System::new();
    /// let mut display = Display::new();
    /// 
    /// display.update(system);
    /// ```
    pub fn update(&mut self, sys: &System) {
        let mut changes = false;
        for i in 0xF00..=0xFFF {
            let byte = sys.memory.get(i);
            let pos = i - 0xF00;
            let y = pos / (sys.screen_width as u16 / 8);
            let x = (pos % (sys.screen_width as u16 / 8))*8;
            for j in 0..8 {
                if (byte & (1 << (7-j))) > 0 {
                    if self.pixels[y as usize][(x + j) as usize] == 0 {
                        changes = true;
                    }
                    self.pixels[y as usize][(x + j) as usize] = 4;
                }
                else if self.pixels[y as usize][(x + j) as usize] > 0 {
                    self.pixels[y as usize][(x + j) as usize] -= 1;
                    if self.pixels[y as usize][(x + j) as usize] == 0 {
                        changes = true;
                    }
                }
            }
        }

        if changes {
            self.render();
        }
    }

    /// Renders the current state of the `pixels` matrix to the console. Called by the `update(...)` method.
    fn render(&self) {
        for y in 0..34 {
            if y == 0 || y == 33 {
                print!("{}[{};{}H", 27 as char, y + 1, 1);
                for x in 0..130 {
                    let c = match x {
                        0 => match y {
                            0 => '╔',
                            33 => '╚',
                            _ => 'Y',
                        },
                        129 => match y {
                            0 => '╗',
                            33 => '╝',
                            _ => 'X',
                        },
                        _ => '═',
                    };
                    print!("{}", c);
                }
                continue;
            }

            print!("{}[{};{}H", 27 as char, y + 1, 1);

            for x in 0..66 {
                
                
                let c = match x {
                    0 => '║',
                    65 => '║',
                    _ => {
                        let pixel = self.pixels[y - 1][x - 1];
                        match pixel {
                            0 => ' ',
                            _ => '█',
                        }
                    },
                };

                if x == 0 || x == 65 {
                    print!("{}", c);
                }
                else {
                    print!("{}", c);
                    print!("{}", c);
                }
                
                
            }
        }
        println!("{}[{};{}H", 27 as char, 36, 0);
    }

}

/// A struct representing the state of a CHIP-8 processor and its peripherals.
pub struct System {
    pub memory: Memory,
    pub registers: Registers,
    pub stack: Stack,
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    pub keyboard: Keyboad,

    pub rng: ThreadRng,

    pub pc: u16,
    pub screen_width: u8,
    pub screen_height: u8,
    loop_frequency: u16,
}

impl System {

    /// Creates a new instance of the `System` struct.
    /// 
    /// Sub-structs are initialized as empty, using their individual `new()` methods.  
    /// 
    /// # Example
    /// ```
    /// let sys = System::new();
    /// ```
    /// 
    pub fn new() -> System {
        System { 
            memory: Memory::new(),
            registers: Registers::new(),
            stack: Stack::new(),
            delay_timer: Timer::new(),
            sound_timer: Timer::new(),
            keyboard: Keyboad::new(),
            rng: rand::thread_rng(),            
            pc: 0,
            screen_width: 64,
            screen_height: 32,
            loop_frequency: 700
        }
    }

    /// Loads a program into the system's main memory.
    /// 
    /// The loaded program's address space starts at 0x200, and its PC is initialized to 0x200.
    /// 
    /// # Example
    /// ```
    /// let mut sys = System::new();
    /// let program = Program::load("path");
    /// sys.load(program);
    /// ```
    pub fn load(&mut self, program: program::Program) {
        for (idx, instr) in program.instructions.iter().enumerate() {
            self.memory.store(0x200 + idx as u16, *instr as u8);
        }
        self.pc = 0x200;
    }

    /// Increments the CHIP-8's PC by two.
    /// 
    /// ' Example
    /// ```
    /// let mut sys = System::new();
    /// sys.increment_pc();
    /// ```
    pub fn increment_pc(&mut self) {
        self.pc += 2;
    }

    /// Starts running the CHIP-8's fetch/decode/execute loop.
    /// 
    /// A mutable reference to a `Display` instance needs to be passed to update the display rendering with each step.
    /// The loop's refresh rate is defined by the `loop_frequency` field. Each step in the loop consists of the following steps, in order:
    /// - Update timers
    /// - Check keyboardinput
    /// - Fetch next instruction
    /// - Increment PC
    /// - Decode & execute instruction
    /// - Update 
    /// 
    /// # Example
    /// ```
    /// let mut sys = System::new();
    /// let mut display = Display::new();
    /// let program = Program::load("path");
    /// 
    /// sys.load(program);
    /// sys.run(&mut display);
    /// ```
    pub fn run(&mut self, display: &mut Display) {
        let delay = 1000000u64/self.loop_frequency as u64;
        loop {
            self.delay_timer.update();
            self.sound_timer.update();
            self.keyboard.update();

            //Fetch
            let op1 = self.memory.get(self.pc);
            let op2 = self.memory.get(self.pc + 1);
            self.increment_pc();

            //Decode & Execute
            if op1 == 0 && op2 == 0 {
                break;
            }
            let op: Instruction = big_endian_8_2(op1, op2).into();
            op.execute(self);

            //Display updates
            display.update(self);            

            //frequency
            thread::sleep(Duration::from_micros(delay));
        }
        println!("CHIP-8 Finished!");
    }
}