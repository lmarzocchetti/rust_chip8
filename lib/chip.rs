use std::{collections::HashMap, fs::File, io::Read, panic::panic_any};

use crate::display;

mod stack;

const RAM_SIZE: usize = 4096;
const PROGRAM_START: u16 = 512;

fn initialize_registers() -> HashMap<u8, u8> {
    let mut hm: HashMap<u8, u8> = HashMap::new();

    hm.insert(0x0, 0);
    hm.insert(0x1, 0);
    hm.insert(0x2, 0);
    hm.insert(0x3, 0);
    hm.insert(0x4, 0);
    hm.insert(0x5, 0);
    hm.insert(0x6, 0);
    hm.insert(0x7, 0);
    hm.insert(0x8, 0);
    hm.insert(0x9, 0);
    hm.insert(0xA, 0);
    hm.insert(0xB, 0);
    hm.insert(0xC, 0);
    hm.insert(0xD, 0);
    hm.insert(0xE, 0);
    hm.insert(0xF, 0);

    hm
}

#[derive(Debug)]
pub struct Chip {
    // Only 12 bits are used in program_counter and index_register
    program_counter: u16,
    index_register: u16,
    registers: HashMap<u8, u8>,

    memory: [u8; RAM_SIZE],
    // stack of addresses of 12 bits, represented as 16 bits
    #[allow(dead_code)]
    stack: stack::Stack<u16>,
    #[allow(dead_code)]
    delay_timer: u8,
    #[allow(dead_code)]
    sound_timer: u8,
    screen: display::Display,
}

fn initialize_font() -> [u8; 4096] {
    let mut arr = [0; 4096];

    let font: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0,
        0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
        0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0,
        0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
        0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0,
        0xF0, 0x80, 0xF0, 0x80, 0x80,
    ];

    for (index, val) in font.iter().enumerate() {
        arr[index] = *val;
    }

    arr
}

impl Default for Chip {
    fn default() -> Self {
        Chip::new()
    }
}

impl Chip {
    pub fn new() -> Self {
        Chip {
            program_counter: 0,
            index_register: 0,
            registers: initialize_registers(),
            memory: initialize_font(),
            stack: stack::Stack::new(),
            delay_timer: 60,
            sound_timer: 60,
            screen: display::Display::new(),
        }
    }

    pub fn load_program(&mut self, filename: &str) {
        let mut f = File::open(filename)
            .expect(format!("Error: Cannot load this file: {}", filename).as_str());

        let mut buf: Vec<u8> = vec![];

        File::read_to_end(&mut f, &mut buf).unwrap();

        for (i, val) in buf.iter().enumerate() {
            self.memory[PROGRAM_START as usize + i] = *val;
        }

        self.program_counter = PROGRAM_START;
    }

    fn fetch(&mut self) -> u16 {
        let mut istr: u16 = self.memory[self.program_counter as usize] as u16;
        istr = (istr << 8) | (self.memory[(self.program_counter + 1) as usize] as u16);
        self.program_counter += 2;

        istr
    }

    // opcode: 00E0
    fn clear_screen(&mut self) {
        self.screen.clear_screen()
    }

    // opcode: 00EE
    fn subroutine_return(&mut self) {}

    // opcode: 1NNN
    fn jump(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }

    // opcode: 6XNN
    fn set(&mut self, second_nibble: u8, third_nibble: u8, fourth_nibble: u8) {
        let val: u8 = (third_nibble << 4) | fourth_nibble;
        *self.registers.get_mut(&second_nibble).unwrap() = val;
    }

    // opcode: 7XNN
    fn add(&mut self, second_nibble: u8, third_nibble: u8, fourth_nibble: u8) {
        let val: u8 = (third_nibble << 4) | fourth_nibble;
        *self.registers.get_mut(&second_nibble).unwrap() += val;
    }

    // opcode: ANNN
    fn set_index(&mut self, second_nibble: u8, third_nibble: u8, fourth_nibble: u8) {
        let mut val: u16 = second_nibble as u16;
        val = (val << 4) | third_nibble as u16;
        val = (val << 4) | fourth_nibble as u16;
        self.index_register = val;
    }

    // opcode: DXYN
    fn display(&mut self, second_nibble: u8, third_nibble: u8, fourth_nibble: u8) {
        *self.registers.get_mut(&0xF).unwrap() = 0;

        for byte in 0..fourth_nibble {
            let y = (self.registers.get(&third_nibble).unwrap() + byte) % 32;
            for bit in 0..8 {
                let x = (self.registers.get(&second_nibble).unwrap() + bit) % 64;
                let color =
                    (self.memory[self.index_register as usize + byte as usize] >> (7 - bit)) & 1;
                if color == 1 && self.screen.get_pixel(x as usize, y as usize) {
                    self.screen.set_pixel(x as usize, y as usize, false);
                    *self.registers.get_mut(&0xF).unwrap() = 1;
                } else if color == 1 && !self.screen.get_pixel(x as usize, y as usize) {
                    self.screen.set_pixel(x as usize, y as usize, true);
                }
            }
        }
        self.screen.display_terminal();
    }

    pub fn instruction(&mut self) {
        let istr = self.fetch();

        // Extract nibbles
        let first_nibble: u8 = (istr >> 12) as u8;
        let second_nibble: u8 = ((istr >> 8) & 0x000F) as u8;
        let third_nibble: u8 = ((istr >> 4) & 0x000F) as u8;
        let fourth_nibble: u8 = (istr & 0x000F) as u8;

        let nnn: u16 = istr & 0x0FFF;

        match first_nibble {
            0x0 => match fourth_nibble {
                0x0 => self.clear_screen(),
                0xE => self.subroutine_return(),
                _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
            },
            0x1 => self.jump(nnn),
            0x6 => self.set(second_nibble, third_nibble, fourth_nibble),
            0x7 => self.add(second_nibble, third_nibble, fourth_nibble),
            0xA => self.set_index(second_nibble, third_nibble, fourth_nibble),
            0xD => self.display(second_nibble, third_nibble, fourth_nibble),
            _ => (),
        }
    }

    pub fn interpret(&mut self) {
        loop {
            self.instruction();
        }
    }
}
