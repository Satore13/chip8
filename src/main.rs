use std::fs::File;
use std::io::Read;
mod chip8;
use chip8::Chip8;

fn main() {
    let mut rom = File::open("roms/PONG").expect("error opening file!");
    let mut brom: Vec<u8> = Vec::new();
    rom.read_to_end(&mut brom).expect("couldn't read file");

    let mut f_rom = [0u8; 0x1000 - 0x200];
    (&mut f_rom[..brom.len()]).copy_from_slice(&brom[..]);

    let mut chip = Chip8::new(Some(f_rom));
    chip.auto_run();

    let ms = [[0;0];0];//chip.screen_memory;
    for row in &ms[..]
    {
        for bit in &row[..]
        {
            print!("{}", *bit as u8);
        }
        println!("");
    }
}
