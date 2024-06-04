use std::{
    collections::HashMap, fs::File, io::Read, panic::panic_any, process::exit, time::Duration,
};

use rand::Rng;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

use crate::display;

mod stack;

enum InstrVersion {
    Old,
    New,
}

const RAM_SIZE: usize = 4096;
const PROGRAM_START: u16 = 512;

const OP_8XY6_VERSION: InstrVersion = InstrVersion::Old;

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

#[derive(Debug, PartialEq)]
enum MetaInputs {
    PressedInput,
    Pass,
}

pub struct Chip {
    // Only 12 bits are used in program_counter and index_register
    program_counter: u16,
    index_register: u16,
    registers: HashMap<u8, u8>,
    memory: [u8; RAM_SIZE],
    // stack of addresses of 12 bits, represented as 16 bits
    stack: stack::Stack<u16>,
    delay_timer: u8,
    num_instructions: u32,
    sound_timer: u8,
    screen: display::Display,
    key_pressed: Option<u8>,
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
            delay_timer: 0,
            num_instructions: 0,
            sound_timer: 0,
            screen: display::Display::new(),
            key_pressed: None,
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
    fn subroutine_return(&mut self) {
        self.program_counter = self.stack.pop();
    }

    // opcode: 1NNN
    fn jump(&mut self, nnn: u16) {
        self.program_counter = nnn;
    }

    // opcode: 2NNN
    fn subroutine_call(&mut self, nnn: u16) {
        self.stack.push(self.program_counter);
        self.program_counter = nnn;
    }

    // opcode: 3XNN
    fn skip_equal_unary(&mut self, second_nibble: u8, nn: u8) {
        let reg_val = *self.registers.get(&second_nibble).unwrap();
        if reg_val == nn {
            self.program_counter += 2;
        }
    }

    // opcode: 4XNN
    fn skip_not_equal_unary(&mut self, second_nibble: u8, nn: u8) {
        let reg_val = *self.registers.get(&second_nibble).unwrap();
        if reg_val != nn {
            self.program_counter += 2;
        }
    }

    // opcode: 5XY0
    fn skip_equal_binary(&mut self, second_nibble: u8, third_nibble: u8) {
        let x = self.registers.get(&second_nibble).unwrap();
        let y = self.registers.get(&third_nibble).unwrap();

        if x == y {
            self.program_counter += 2;
        }
    }

    // opcode: 6XNN
    fn set_value(&mut self, second_nibble: u8, nn: u8) {
        *self.registers.get_mut(&second_nibble).unwrap() = nn;
    }

    // opcode: 7XNN
    fn add_noncarry(&mut self, second_nibble: u8, nn: u8) {
        *self.registers.get_mut(&second_nibble).unwrap() =
            self.registers.get(&second_nibble).unwrap().wrapping_add(nn);
    }

    // opcode: 8XY0
    fn set_registers(&mut self, second_nibble: u8, third_nibble: u8) {
        *self.registers.get_mut(&second_nibble).unwrap() =
            *self.registers.get(&third_nibble).unwrap();
    }

    // opcode: 8XY1
    fn binary_or(&mut self, second_nibble: u8, third_nibble: u8) {
        *self.registers.get_mut(&second_nibble).unwrap() |=
            *self.registers.get(&third_nibble).unwrap();
    }

    // opcode: 8XY2
    fn binary_and(&mut self, second_nibble: u8, third_nibble: u8) {
        *self.registers.get_mut(&second_nibble).unwrap() &=
            *self.registers.get(&third_nibble).unwrap();
    }

    // opcode: 8XY3
    fn logical_xor(&mut self, second_nibble: u8, third_nibble: u8) {
        *self.registers.get_mut(&second_nibble).unwrap() ^=
            *self.registers.get(&third_nibble).unwrap();
    }

    // opcode: 8XY4
    fn add_carry(&mut self, second_nibble: u8, third_nibble: u8) {
        let operation_res: u16 = *self.registers.get(&second_nibble).unwrap() as u16
            + *self.registers.get(&third_nibble).unwrap() as u16;

        *self.registers.get_mut(&second_nibble).unwrap() = operation_res as u8;

        *self.registers.get_mut(&0xF).unwrap() = if operation_res > 0xFF { 1 } else { 0 };
    }

    // opcode: 8XY5
    fn subtract_vx(&mut self, second_nibble: u8, third_nibble: u8) {
        let (new_vx, borrow) = self
            .registers
            .get_mut(&second_nibble)
            .unwrap()
            .overflowing_sub(*self.registers.get(&third_nibble).unwrap());
        let new_vf: u8 = if borrow { 0 } else { 1 };

        *self.registers.get_mut(&second_nibble).unwrap() = new_vx;
        *self.registers.get_mut(&0xF).unwrap() = new_vf;
    }

    // TODO: Far configurare dall'utente se non
    //       vuole questa config: "Set VX to the value of VY"

    // opcode: 8XY6
    fn shift_right(&mut self, second_nibble: u8, third_nibble: u8) {
        match OP_8XY6_VERSION {
            InstrVersion::Old => {
                *self.registers.get_mut(&second_nibble).unwrap() =
                    *self.registers.get(&third_nibble).unwrap();

                let shifted_value = *self.registers.get(&second_nibble).unwrap() & 0b00000001;
                *self.registers.get_mut(&second_nibble).unwrap() =
                    *self.registers.get(&second_nibble).unwrap() >> 1;

                if shifted_value == 1 {
                    *self.registers.get_mut(&0xF).unwrap() = 1;
                } else {
                    *self.registers.get_mut(&0xF).unwrap() = 0;
                }
            }
            InstrVersion::New => {
                *self.registers.get_mut(&0xF).unwrap() =
                    *self.registers.get(&second_nibble).unwrap() & 1;

                *self.registers.get_mut(&second_nibble).unwrap() >>= 1;
            }
        }
    }

    // opcode: 8XYE
    fn shift_left(&mut self, second_nibble: u8, third_nibble: u8) {
        match OP_8XY6_VERSION {
            InstrVersion::Old => {
                *self.registers.get_mut(&second_nibble).unwrap() =
                    *self.registers.get(&third_nibble).unwrap();

                let shifted_value =
                    (*self.registers.get(&second_nibble).unwrap() & 0b10000000) >> 7;
                *self.registers.get_mut(&second_nibble).unwrap() =
                    *self.registers.get(&second_nibble).unwrap() << 1;

                if shifted_value == 1 {
                    *self.registers.get_mut(&0xF).unwrap() = 1;
                } else {
                    *self.registers.get_mut(&0xF).unwrap() = 0;
                }
            }
            InstrVersion::New => {
                *self.registers.get_mut(&0xF).unwrap() =
                    (*self.registers.get(&second_nibble).unwrap() & 0b10000000) >> 7;

                *self.registers.get_mut(&second_nibble).unwrap() <<= 1;
            }
        }
    }

    // opcode: 8XY7
    fn subtract_vy(&mut self, second_nibble: u8, third_nibble: u8) {
        let (new_vx, borrow) = self
            .registers
            .get_mut(&third_nibble)
            .unwrap()
            .overflowing_sub(*self.registers.get(&second_nibble).unwrap());
        let new_vf: u8 = if borrow { 0 } else { 1 };

        *self.registers.get_mut(&second_nibble).unwrap() = new_vx;
        *self.registers.get_mut(&0xF).unwrap() = new_vf;
    }

    // opcode: 9XY0
    fn skip_not_equal_binary(&mut self, second_nibble: u8, third_nibble: u8) {
        let x = self.registers.get(&second_nibble).unwrap();
        let y = self.registers.get(&third_nibble).unwrap();

        if x != y {
            self.program_counter += 2;
        }
    }

    // opcode: ANNN
    fn set_index(&mut self, nnn: u16) {
        self.index_register = nnn;
    }

    // TODO: Implemented the classic behaviour: make this configurable
    //       to work as BXNN -> jump to the address XNN + register VX

    // opcode: BNNN
    fn jump_with_offset(&mut self, nnn: u16) {
        self.program_counter = nnn + *self.registers.get(&0x0).unwrap() as u16;
    }

    // opcode: CXNN
    fn random(&mut self, second_nibble: u8, nn: u8) {
        let random: u8 = rand::thread_rng().gen();
        *self.registers.get_mut(&second_nibble).unwrap() = random & nn;
    }

    // opcode: DXYN
    fn display(&mut self, second_nibble: u8, third_nibble: u8, fourth_nibble: u8) {
        let x_coord = *self.registers.get(&second_nibble).unwrap();
        let y_coord = *self.registers.get(&third_nibble).unwrap();

        let num_rows = fourth_nibble;

        let mut flipped = false;

        for y_line in 0..num_rows {
            let addr = self.index_register + y_line as u16;
            let pixels = self.memory[addr as usize];

            for x_line in 0..8 {
                if (pixels & (0b1000_0000 >> x_line)) != 0 {
                    let x = (x_coord as usize + x_line) % 64;
                    let y = (y_coord as usize + y_line as usize) % 32;

                    let idx = x + 64 * y;
                    flipped |= self.screen.data[idx];
                    self.screen.data[idx] ^= true;
                }
            }
        }

        if flipped {
            *self.registers.get_mut(&0xF).unwrap() = 1;
        } else {
            *self.registers.get_mut(&0xF).unwrap() = 0;
        }

        self.screen.redraw = true;
    }

    // opcode: EX9E
    fn skip_if_key_pressed(&mut self, second_nibble: u8) {
        let key = *self.registers.get(&second_nibble).unwrap();

        match self.key_pressed {
            Some(a) => {
                if key == a {
                    self.program_counter += 2;
                }
            }
            None => {}
        }
    }

    // opcode: EXA1
    fn skip_if_key_not_pressed(&mut self, second_nibble: u8) {
        let key = *self.registers.get(&second_nibble).unwrap();

        match self.key_pressed {
            Some(a) => {
                if key != a {
                    self.program_counter += 2;
                }
            }
            None => {
                self.program_counter += 2;
            }
        }
    }

    // opcode: FX07
    fn set_reg_to_delay(&mut self, second_nibble: u8) {
        *self.registers.get_mut(&second_nibble).unwrap() = self.delay_timer;
    }

    // opcode: FX0A
    fn get_key(&mut self, second_nibble: u8) {
        loop {
            _ = self.poll_inputs();
            if self.key_pressed.is_some() {
                *self.registers.get_mut(&second_nibble).unwrap() = self.key_pressed.unwrap();
                break;
            }
        }
    }

    // opcode: FX15
    fn set_delay_to_reg(&mut self, second_nibble: u8) {
        self.delay_timer = *self.registers.get(&second_nibble).unwrap();
    }

    // opcode: FX18
    fn set_sound_to_vx(&mut self, second_nibble: u8) {
        self.sound_timer = *self.registers.get(&second_nibble).unwrap();
    }

    // opcode: FX1E
    fn add_to_index(&mut self, second_nibble: u8) {
        self.index_register += *self.registers.get(&second_nibble).unwrap() as u16;
        *self.registers.get_mut(&0xF).unwrap() = if self.index_register > 0x0F00 { 1 } else { 0 };
    }

    // opcode: FX29
    fn font_character(&mut self, second_nibble: u8) {
        self.index_register = (*self.registers.get(&second_nibble).unwrap() & 0xF) as u16 * 5;
    }

    // opcode: FX33
    fn binary_coded_dec_conv(&mut self, second_nibble: u8) {
        let num = *self.registers.get(&second_nibble).unwrap();
        self.memory[self.index_register as usize + 2] = num % 10;
        self.memory[self.index_register as usize + 1] = (num % 100) / 10;
        self.memory[self.index_register as usize] = num / 100;
    }

    // TODO: Implement the user choice for old behaviour of this instruction

    // opcode: FX55
    fn store_memory(&mut self, second_nibble: u8) {
        for reg in 0..=second_nibble {
            self.memory[(self.index_register + reg as u16) as usize] =
                *self.registers.get(&reg).unwrap();
        }
        self.index_register += second_nibble as u16 + 1;
    }

    // opcode: FX65
    fn load_memory(&mut self, second_nibble: u8) {
        for reg in 0..=second_nibble {
            *self.registers.get_mut(&reg).unwrap() =
                self.memory[(self.index_register + reg as u16) as usize];
        }
        self.index_register += second_nibble as u16 + 1;
    }

    pub fn instruction(&mut self) {
        let istr = self.fetch();

        // Extract nibbles
        let first_nibble: u8 = (istr >> 12) as u8;
        let second_nibble: u8 = ((istr >> 8) & 0x000F) as u8;
        let third_nibble: u8 = ((istr >> 4) & 0x000F) as u8;
        let fourth_nibble: u8 = (istr & 0x000F) as u8;

        let nn: u8 = (istr & 0x00FF) as u8;
        let nnn: u16 = istr & 0x0FFF;

        match first_nibble {
            0x0 => match second_nibble {
                0x0 => match third_nibble {
                    0xE => match fourth_nibble {
                        0x0 => self.clear_screen(),
                        0xE => self.subroutine_return(),
                        _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
                    },
                    _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
                },
                _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
            },
            0x1 => self.jump(nnn),
            0x2 => self.subroutine_call(nnn),
            0x3 => self.skip_equal_unary(second_nibble, nn),
            0x4 => self.skip_not_equal_unary(second_nibble, nn),
            0x5 => self.skip_equal_binary(second_nibble, third_nibble),
            0x6 => self.set_value(second_nibble, nn),
            0x7 => self.add_noncarry(second_nibble, nn),
            0x8 => match fourth_nibble {
                0x0 => self.set_registers(second_nibble, third_nibble),
                0x1 => self.binary_or(second_nibble, third_nibble),
                0x2 => self.binary_and(second_nibble, third_nibble),
                0x3 => self.logical_xor(second_nibble, third_nibble),
                0x4 => self.add_carry(second_nibble, third_nibble),
                0x5 => self.subtract_vx(second_nibble, third_nibble),
                0x6 => self.shift_right(second_nibble, third_nibble),
                0x7 => self.subtract_vy(second_nibble, third_nibble),
                0xE => self.shift_left(second_nibble, third_nibble),
                _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
            },
            0x9 => self.skip_not_equal_binary(second_nibble, third_nibble),
            0xA => self.set_index(nnn),
            0xB => self.jump_with_offset(nnn),
            0xC => self.random(second_nibble, nn),
            0xD => self.display(second_nibble, third_nibble, fourth_nibble),
            0xE => match third_nibble {
                0x9 | 0xA => match fourth_nibble {
                    0x1 => self.skip_if_key_not_pressed(second_nibble),
                    0xE => self.skip_if_key_pressed(second_nibble),
                    _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
                },
                _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
            },
            0xF => match third_nibble {
                0x0 => match fourth_nibble {
                    0x7 => self.set_reg_to_delay(second_nibble),
                    0xA => self.get_key(second_nibble),
                    _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
                },
                0x1 => match fourth_nibble {
                    0x5 => self.set_delay_to_reg(second_nibble),
                    0x8 => self.set_sound_to_vx(second_nibble),
                    0xE => self.add_to_index(second_nibble),
                    _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
                },
                0x2 => self.font_character(second_nibble),
                0x3 => self.binary_coded_dec_conv(second_nibble),
                0x5 => self.store_memory(second_nibble),
                0x6 => self.load_memory(second_nibble),
                _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
            },
            _ => panic_any(format!("Error: Instruction {:#06x} do not exists!", istr)),
        }
    }

    fn poll_inputs(&mut self) -> MetaInputs {
        for event in self.screen.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    exit(0);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => self.key_pressed = Some(0x1),
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => self.key_pressed = Some(0x2),
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => self.key_pressed = Some(0x3),
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => self.key_pressed = Some(0xC),
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => self.key_pressed = Some(0x4),
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => self.key_pressed = Some(0x5),
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => self.key_pressed = Some(0x6),
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => self.key_pressed = Some(0xD),
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => self.key_pressed = Some(0x7),
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => self.key_pressed = Some(0x8),
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => self.key_pressed = Some(0x9),
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => self.key_pressed = Some(0xE),
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => self.key_pressed = Some(0xA),
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => self.key_pressed = Some(0x0),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => self.key_pressed = Some(0xB),
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => self.key_pressed = Some(0xF),
                Event::KeyUp { .. } => {
                    self.key_pressed = None;
                }
                _ => return MetaInputs::Pass,
            }
            return MetaInputs::PressedInput;
        }

        return MetaInputs::Pass;
    }

    pub fn interpret(&mut self) {
        loop {
            if self.screen.redraw {
                self.screen.redraw = false;
                self.screen.canvas.set_draw_color(Color::BLACK);
                self.screen.canvas.clear();

                self.screen.canvas.set_draw_color(Color::WHITE);
                self.screen
                    .canvas
                    .fill_rects(&self.screen.create_white_rects())
                    .expect("error printing the squares");

                self.screen.canvas.present();
            }

            _ = self.poll_inputs();

            if self.num_instructions == 12 {
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }
                self.num_instructions = 0;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }

            self.instruction();
            self.num_instructions += 1;

            // self.screen.canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 720));
        }
    }
}
