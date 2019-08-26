#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::Read;
use argparse::{ArgumentParser, Store};

mod chip8;
use chip8::{Chip8, Result, ResultExt};

fn load_rom(rom_name: String) -> Result<[u8; 0xE00]>
{
    let mut rom = File::open(&rom_name)?;
    let mut brom: Vec<u8> = Vec::new();
    rom.read_to_end(&mut brom)?;
    let mut f_rom = [0u8; 0xE00];
    if brom.len() > f_rom.len()
    {
        bail!(format!("Rom can't be bigger than {}B and {} is {}B",
                                        f_rom.len(), rom_name , brom.len()));
    }
    (&mut f_rom[..brom.len()]).copy_from_slice(&brom[..]);
    Ok(f_rom)
}


fn main()
{
    if let Err(e) = run()
    {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace()
        {
            println!("backtrace: {:?}", backtrace);
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()>
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

    let rom = load_rom(rom_name).chain_err(|| "Error loading rom")?;
    let mut chip = Chip8::new(Some(rom)).chain_err(|| "Error creating Chip8 struct")?;
    chip.auto_run().chain_err(|| "Error executing rom")?;
    Ok(())
}
