use std::fs::File;
use std::io::Read;
use argparse::{ArgumentParser, Store};
mod chip8;
use chip8::Chip8;

fn load_rom(rom_name: String) -> Result<[u8; 0xE00], std::io::Error>
{
    let mut rom = File::open(&rom_name)?;
    let mut brom: Vec<u8> = Vec::new();
    rom.read_to_end(&mut brom)?;
    let mut f_rom = [0u8; 0xE00];
    if brom.len() > f_rom.len()
    {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                                format!("Rom can't be bigger than {}B and {} is {}B",
                                        f_rom.len(), rom_name , brom.len())));
    }
    (&mut f_rom[..brom.len()]).copy_from_slice(&brom[..]);
    Ok(f_rom)
}


fn main()
{
    let mut rom_name = String::new();
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Chip-8 interpreter by Satore");
        ap.refer(&mut rom_name)
            .add_argument("ROM", Store,
                "File containing the rom").required();
                ap.parse_args_or_exit();
    }


    match load_rom(rom_name)
    {
        Ok(rom) =>
        {
            let mut chip = Chip8::new(Some(rom));
            chip.auto_run();
        }
        Err(e) => eprintln!("Error reading the rom: \n{}",e),
    }
}
