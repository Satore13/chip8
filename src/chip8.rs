use minifb::{Window, WindowOptions, Key, KeyRepeat};
use std::{time::{Duration, Instant}};
const C8_SCREEN_WIDTH: usize = 64;
const C8_SCREEN_HEIGTH: usize = 32;
const SCALE: minifb::Scale = minifb::Scale::X16;
const KEY_CODES: [(u8, Key);16] =
                                [(0x1, Key::Key1), (0x2, Key::Key2), (0x3, Key::Key3), (0xC, Key::Key4),
                                (0x4, Key::Q), (0x5, Key::W), (0x6, Key::E), (0xD, Key::R),
                                (0x7, Key::A), (0x8, Key::S), (0x9, Key::D), (0xE, Key::F),
                                (0xA, Key::Z), (0x0, Key::X), (0xB, Key::C), (0xF, Key::V),];

fn get_hexcode_from_key(key: Key) -> Option<u8>
{
    for (h, k) in KEY_CODES.iter()
    {
        if key == *k
        {
            return Some(*h);
        }
    }
    None
}
fn get_key_from_hexcode(hexcode: u8) -> Option<Key>
{
    for (h, k) in KEY_CODES.iter()
    {
        if hexcode == *h
        {
            return Some(*k);
        }
    }
    None
}

pub struct Chip8
{
    mem: [u8; 0x1000], // 4096 memory size ; 8bits
    registers: [u8; 0x10], // 16 8bit registers
    program_counter: usize,
    stack_pointer: usize,
    stack: [usize; 0x10],
    index: usize,
    screen_memory: [[bool; C8_SCREEN_WIDTH]; C8_SCREEN_HEIGTH], // 64*32 screen
    dt: u8,
    st: u8,
    window: Window,
    waiting_for_key: Option<u8>,
}

impl Chip8
{
    pub fn new(rom :Option<[u8; 0x1000 - 0x200]>) -> Chip8
    {
        let mut new_chip = Chip8
        {
            mem: [0; 0x1000],
            registers: [0; 0x10],
            program_counter: 0x200,
            stack_pointer: 0x0,
            stack: [0; 0x10],
            index: 0,
            screen_memory: [[false; C8_SCREEN_WIDTH]; C8_SCREEN_HEIGTH],
            dt: 0,
            st: 0,
            window: Window::new("Chip-8 Emulator by Satore",
                                C8_SCREEN_WIDTH, C8_SCREEN_HEIGTH,
                                    WindowOptions
                                    {
                                        borderless: false,
                                        title: true,
                                        resize: false,
                                        scale: SCALE
                                    }).expect("Couldn't create window"),
            waiting_for_key: None,
        };
        let hex_digits = [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                                0x20, 0x60, 0x20, 0x20, 0x70, // 1
                                0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                                0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                                0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                                0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                                0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                                0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                                0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                                0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                                0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                                0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                                0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                                0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                                0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                                0xF0, 0x80, 0xF0, 0x80, 0x80, // F
                                ];
        (&mut new_chip.mem[0..hex_digits.len()]).copy_from_slice(&hex_digits);
        if let Some(urom) = rom
        {
            (&mut new_chip.mem[0x200..]).copy_from_slice(&urom);
        }
        new_chip
    }

    fn get_screen_buffer(&self) -> [u32; C8_SCREEN_WIDTH * C8_SCREEN_HEIGTH]
    {
        let mut buffer: [u32; C8_SCREEN_WIDTH * C8_SCREEN_HEIGTH] =
                                    [0; C8_SCREEN_WIDTH * C8_SCREEN_HEIGTH];
        for (y, row) in self.screen_memory[..].iter().enumerate()
        {
            for (x, bit) in row[..].iter().enumerate()
            {
                let bit = *bit;
                buffer[y*C8_SCREEN_WIDTH + x] = match bit
                                                {
                                                    true =>  0xFFFFFFFF,
                                                    false => 0x00000000,
                                                };
            }
        }
        buffer
    }

    pub fn auto_run(&mut self) 
    {
        let mut previous_draw_instant = Instant::now();
        let mut previous_update_instant = Instant::now();
        while self.window.is_open()
        {
            //Update at 500hz
            if Instant::now().duration_since(previous_update_instant) > Duration::from_millis(1)
            {
                if let Some(keycode) = self.waiting_for_key
                {
                    if let Some(keys) = self.window.get_keys_pressed(KeyRepeat::No)
                    {
                        if let Some(key) = (*keys).iter().next()
                        {
                            if let Some(hexcode) = get_hexcode_from_key(*key)
                            {
                                self.registers[keycode as usize] = hexcode;
                                self.waiting_for_key = None;
                            }
                        }
                    }
                }
                else
                {

                    let instruction = self.fetch_instruction();
                    self.program_counter += 2;
                    self.execute_instruction(instruction);
                }
                previous_update_instant = Instant::now();
            }

            // Draw at 60Hz
            if Instant::now().duration_since(previous_draw_instant) > Duration::from_millis(1000/60)
            {
                self.dt = if self.dt > 0 { self.dt - 1} else { 0 };
                self.st = if self.st > 0 { self.st - 1} else { 0 };
                self.window.update_with_buffer(&self.get_screen_buffer()).expect("Couldn't update screen");
                previous_draw_instant = Instant::now();
            }
        }
    }
    #[allow(dead_code)]
    pub fn print_debug(&self)
    {
        println!("Chip8 {{ registers:{:X?} program_counter:{:X?} stack_pointer:{:X?} stack:{:X?} index:{:X?}}} dt:{}, st:{}",
                    self.registers, self.program_counter, self.stack_pointer, self.stack, self.index, self.dt, self.st);
    }
    #[allow(dead_code)]
    pub fn print_slice_mem(&self, a: usize, b: usize)
    {
        for (index, value) in (&self.mem[a..(b + a)]).iter().enumerate()
        {
                println!("{:#X}: {:#X}", index + a, value);
        }
    }
    fn fetch_instruction(&self) -> u16
    {
        let bit1 = self.mem[self.program_counter as usize];
        let bit2 = self.mem[(self.program_counter + 1) as usize];
        ((bit1 as u16) << 8) | bit2 as u16
    }

    fn get_addr(n1: u8, n2: u8, n3: u8) -> usize
    {
        ((n1 as usize) << 8) |
        ((n2 as usize) << 4) |
        (n3 as usize)
    }

    fn get_kk(k1: u8, k2: u8) -> u8
    {
        (k1 << 4) | k2
    }

    fn execute_instruction(&mut self, preinstruction: u16)
    {
        //println!("{:X}", preinstruction);
        //self.print_debug();
        let instruction =
        {
            (((preinstruction & 0xF000) >> 12) as u8,
            ((preinstruction & 0x0F00) >> 8) as u8,
            ((preinstruction & 0x00F0) >> 4) as u8,
            (preinstruction & 0x000F) as u8)
        };
        match instruction
        {
            //00E0: CLS
            (0, 0, 0xE, 0) =>
            {
                self.screen_memory = [[false; C8_SCREEN_WIDTH]; C8_SCREEN_HEIGTH];
            }
            //00EE: RET
            (0, 0, 0xE, 0xE) =>
            {
                self.program_counter = self.stack[self.stack_pointer];
                self.stack_pointer -= 1;
            }
            //1nnn: JP addr
            (1, n1, n2, n3) =>
            {
                self.program_counter = Chip8::get_addr(n1, n2, n3) as usize;
            }
            //2nnn: CALL addr
            (2, n1, n2, n3) =>
            {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer] = self.program_counter;
                self.program_counter = Chip8::get_addr(n1, n2, n3) as usize;
            }
            //3xkk SE Vx, byte
            (3, x, k1, k2) =>
            {
                if self.registers[x as usize] == Chip8::get_kk(k1, k2)
                    {self.program_counter += 2;}
            }
            //4xkk SNE Vx, byte
            (4, x, k1, k2) =>
            {
                if self.registers[x as usize] != Chip8::get_kk(k1, k2)
                    {self.program_counter += 2;}
            }
            //5xy0 SE Vx, Vy
            (5, x, y, 0) =>
            {
                if self.registers[x as usize] == self.registers[y as usize]
                    {self.program_counter += 2;}
            }
            //6xkk LD Vx, byte
            (6, x, k1, k2) =>
            {
                self.registers[x as usize] = Chip8::get_kk(k1, k2);
            }
            //7xkk ADD Vx, byte
            (7, x, k1, k2) =>
            {
                self.registers[x as usize] =
                    self.registers[x as usize].wrapping_add(Chip8::get_kk(k1, k2));
            }
            //8xy0 LD Vx, Vy
            (8, x, y, 0) =>
            {
                self.registers[x as usize] = self.registers[y as usize];
            }
            //8xy1 OR Vx, Vy
            (8, x, y, 1) =>
            {
                self.registers[x as usize] |= self.registers[y as usize];
            }
            //8xy2 AND Vx, Vy
            (8, x, y, 2) =>
            {
                self.registers[x as usize] &= self.registers[y as usize];
            }
            //8xy3 XOR Vx, Vy
            (8, x, y, 3) =>
            {
                self.registers[x as usize] ^= self.registers[y as usize];
            }
            //8xy4 ADD Vx, Vy, VF = carry
            (8, x, y, 4) =>
            {
                let r = self.registers[x as usize].overflowing_add(self.registers[y as usize]);
                self.registers[x as usize] = r.0;
                self.registers[0xF] = r.1 as u8;
            }
            //8xy5 SUB Vx, Vy, VF = NOT borrow (overflow)
            (8, x, y, 5) =>
            {
                self.registers[0xF] =
                    (self.registers[x as usize] > self.registers[y as usize]) as u8;
                self.registers[x as usize] =
                    self.registers[x as usize].wrapping_sub(self.registers[y as usize]);
            }
            //8xy6 SHR Vx
            (8, x, _y, 6) =>
            {
                self.registers[0xF] = (self.registers[x as usize] & 1 == 1) as u8;
                self.registers[x as usize] >>= 1;
            }
            //8xy7 SUBN Vx, Vy, VF = NOT borrow (overflow)
            (8, x, y, 7) =>
            {
                self.registers[0xF] =
                    (self.registers[y as usize] > self.registers[x as usize]) as u8;
                self.registers[x as usize] =
                    self.registers[y as usize].wrapping_sub(self.registers[x as usize]);
            }
            //8xyE SHL Vx
            (8, x, _y, 0xE) =>
            {
                self.registers[0xF] = (self.registers[x as usize] & 128 == 128) as u8;
                self.registers[x as usize] <<= 1;
            }
            //9xy0 SNE Vx, Vy
            (9, x, y, 0) =>
            {
                if self.registers[x as usize] != self.registers[y as usize]
                    { self.program_counter += 2; }
            }
            //Annn LD I, addr
            (0xA, n1, n2, n3) =>
            {
                self.index = Chip8::get_addr(n1, n2, n3);
            }
            //Bnnn JP V0, addr
            (0xB, n1, n2, n3) =>
            {
                self.program_counter = Chip8::get_addr(n1, n2, n3) + self.registers[0x0] as usize;
            }
            //Cxkk RND Vx, byte
            (0xC, x, k1, k2) =>
            {
                self.registers[x as usize] =
                    rand::random::<u8>() & Chip8::get_kk(k1, k2);
            }
            //Dxyn DRw Vx, Vy, nibble
            (0xD, x, y, n) =>
            {
                // Get the x and y values from the registers
                let initial_x = self.registers[x as usize] as usize;
                let initial_y = self.registers[y as usize] as usize;

                let mut flag: u8 = 0;

                // Iterate through rows (index) to (index + n) byte index indicates the byte from
                // memory which will drawn
                for (y_offset, byte_index) in (self.index..(n as usize + self.index)).enumerate()
                {
                    // Iterate through the bits in the sprite byte
                    let byte = BitIteratoru8::new(self.mem[byte_index]);
                    for (x_offset, bit) in byte.enumerate()
                    {
                        // Adding the offset to the initial coords
                        let x: usize = (initial_x + x_offset) % C8_SCREEN_WIDTH as usize;
                        let y: usize = (initial_y + y_offset) % C8_SCREEN_HEIGTH as usize;
                        // If bits are overlapped, set VF to 1
                        if self.screen_memory[y][x] & bit
                        {
                            flag = 1;
                        }
                        self.screen_memory[y][x] ^= bit;
                    }
                }
                self.registers[0xF] = flag;
            }
            //Ex9E SKP Vx
            (0xE, x, 0x9, 0xE) =>
            {
                if let Some(key) = get_key_from_hexcode(self.registers[x as usize])
                {
                    if self.window.is_key_down(key)
                    {
                        self.program_counter += 2;
                    }
                }else
                {
                    eprintln!("Instruction: {:X}, Hexcode {:X} in Register: V{:X} doesn't correspond to any key"
                                , preinstruction, self.registers[x as usize], x);
                }
            }
            //Ex9E SKNP Vx
            (0xE, x, 0xA, 0x1) =>
            {
                if let Some(key) = get_key_from_hexcode(self.registers[x as usize])
                {
                    if !self.window.is_key_down(key)
                    {
                        self.program_counter += 2;
                    }
                }else
                {
                    eprintln!("Instruction: {:X}, Hexcode {:X} in Register: V{:X} doesn't correspond to any key"
                                , preinstruction, self.registers[x as usize], x);
                }
            }
            //Fx07 LD Vx, DT
            (0xF, x, 0x0, 0x7) =>
            {
                self.registers[x as usize] = self.dt;
            }
            //Fx0A LD Vx, KeyPress
            (0xF, x, 0x0, 0xA) =>
            {
                self.waiting_for_key = Some(x);
            }
            //Fx15 LD DT, Vx
            (0xF, x, 0x1, 0x5) =>
            {
                self.dt = self.registers[x as usize];
            }
            //Fx18 LD ST, Vx
            (0xF, x, 0x1, 0x8) =>
            {
                self.st = self.registers[x as usize];
            }
            //Fx1E ADD I, Vx
            (0xF, x, 0x1, 0xE) =>
            {
                self.index = self.index + self.registers[x as usize] as usize;
            }
            //Fx29 LD F, Vx
            (0xF, x, 0x2, 0x9) =>
            {
                self.index = self.registers[x as usize] as usize * 5;
            }
            //Fx33 LD B, Vx
            (0xF, x, 0x3, 0x3) =>
            {
                let hundreds_digit = self.registers[x as usize] / 100;
                let tens_digits = (self.registers[x as usize] - hundreds_digit * 100) / 10;
                let ones_digits = self.registers[x as usize] - tens_digits * 10 - hundreds_digit * 100;
                self.mem[self.index] = hundreds_digit;
                self.mem[self.index + 1] = tens_digits;
                self.mem[self.index + 2] = ones_digits;
            }
            //Fx55 LD [I]. Vx
            (0xF, x, 0x5, 0x5) =>
            {
                for i in 0..=x
                {
                    let i = i as usize;
                    self.mem[i + self.index] = self.registers[i];
                }
            }
            //Fx65 LD Vx, [I]
            (0xF, x, 0x6, 0x5) =>
            {
                for i in 0..=x
                {
                    let i = i as usize;
                    self.registers[i] = self.mem[i + self.index];
                }
            }
            _ =>
            {
                eprintln!("Unrecognized instruction: {:?}", preinstruction);
            }
        }
    }
}

struct BitIteratoru8
{
    count: usize,
    byte: u8,
}
impl BitIteratoru8
{
    pub fn new(byte: u8) -> BitIteratoru8
    {
        BitIteratoru8 {count: 0, byte}
    }
}

impl Iterator for BitIteratoru8
{
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item>
    {
        if self.count > 7
        {
            return None;
        }
        let bit = (self.byte >> (7 - self.count)) & 1;
        self.count += 1;
        Some(bit == 1)
    }
}

#[test]
fn test_encode_instruction_tuple()
{
    assert_eq!(Chip8::get_addr(0xF, 0x2, 0xA), 0xF2A);
    assert_eq!(Chip8::get_kk(0xF, 0xB), 0xFB);
}
