fn main() {
    let mut chip = Chip8::new();
    chip.print_debug();
    chip.execute_instruction(0x700A);
    chip.print_debug();
    chip.execute_instruction(0x8006);
    chip.print_debug();
    chip.execute_instruction(0x8006);
    chip.print_debug();
    chip.execute_instruction(0x8006);
    chip.print_debug();
    chip.print_slice_mem(0, 0x50);
}

struct Chip8
{
    mem: [u8; 0x1000], // 4096 memory size ; 8bits
    registers: [u8; 0x10], // 16 8bit registers
    program_counter: usize,
    stack_pointer: usize,
    stack: [usize; 0x10],
}

impl Chip8
{
    pub fn new() -> Chip8
    {
        let mut new_chip = Chip8
        {
            mem: [0; 0x1000],
            registers: [0; 0x10],
            program_counter: 0x200,
            stack_pointer: 0x0,
            stack: [0; 0x10]
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
        (&mut new_chip.mem[..hex_digits.len()]).copy_from_slice(&hex_digits);
        new_chip
    }
    pub fn print_debug(&self)
    {
        println!("Chip8 {{ registers: {:X?} program_counter: {:X?} stack_pointer: {:X?} stack {:X?} }}",
                    self.registers, self.program_counter, self.stack_pointer, self.stack);
    }
    pub fn print_slice_mem(&self, a: usize, b: usize)
    {
        for (index, value) in (&self.mem[a..b]).iter().enumerate()
        {
                println!("{:#X}: {:#X}", index, value);
        }
    }
    fn fetch_instruction(&self) -> u16
    {
        let bit1 = self.get_mem_value(self.program_counter as usize);
        let bit2 = self.get_mem_value((self.program_counter + 1) as usize);

        (bit1 << 8) as u16 | bit2 as u16

    }

    pub fn get_mem_value(&self, index: usize) -> u8
    {
        self.mem[index]
    }

    fn get_addr(n1: u8, n2: u8, n3: u8) -> usize
    {
        (((n1 as usize) << 8) |
        ((n2 as usize) << 4) |
        (n3 as usize))
    }

    fn get_kk(k1: u8, k2: u8) -> u8
    {
        (k1 << 4) | k2
    }

    fn execute_instruction(&mut self, preinstruction: u16)
    {
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
            (0, 0, 0xE, 0) => (),//TODO: CLEAR_SCREEN

            //00EE: RET
            (0, 0, 0xE, 0xE) =>
            {
                ;
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
            (8, x, y, 6) =>
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
            (8, x, y, 0xE) =>
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
            ()
            _ =>
            {
                println!("Unrecognized instruction: {:?}", preinstruction);
            }
        }
    }
}

#[test]
fn test_encode_instruction_tuple()
{
    assert_eq!(Chip8::get_addr(0xF, 0x2, 0xA), 0xF2A);
    assert_eq!(Chip8::get_kk(0xF, 0xB), 0xFB);
}
