
use std::io;
use std::io::prelude::*;
use std::fs::File;

pub struct Cart {
    pub rom: Vec<u8>,
    has_mapper: bool,
    rom_bank: usize,
}

#[derive(Debug)]
pub struct CartLoadError {
    pub error: String,
}

impl Cart {
    pub fn load(rom_path: &String) -> Result<Cart, CartLoadError> {
        let mut f = try!(File::open(rom_path));
        let mut buffer = Vec::new();

        try!(f.read_to_end(&mut buffer));

        let has_mapper = buffer[0x147] != 0;

        Ok(Cart {
            rom: buffer,
            has_mapper: has_mapper,
            rom_bank: 1,
        })
    }

    pub fn create_from_slice(slice: &[u8]) -> Cart {
        Cart {
            rom: slice.to_vec(),
            has_mapper: slice[0x147]!=0,
            rom_bank: 1,
        }
    }

    // Simple version without mapping. Needs to be enhanced for more complex roms
    pub fn read(&self, address:u16) -> u8 {
        let bank_offset = self.rom_bank*0x4000;
        match address {
            _ if address < 0x4000 => if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0xff },
            _ if address < 0x8000 => self.rom[bank_offset + ((address&0x3fff) as usize)],
            _ => 0xff, //{ println!("Warning: Reading outside the rom!"); 0 }
        }
    }

    pub fn write(&mut self, address:u16, data:u8) {
        if self.has_mapper {
            match address {
                _ if address < 0x2000 => (), //Enable ram
                _ if address < 0x4000 => self.rom_bank = if data==0 {1} else {(data & 0x1f) as usize},
                _ => (), //ToDo!
            }
        }
    }
}


impl From<io::Error> for CartLoadError {
    fn from(err: io::Error) -> CartLoadError {
        CartLoadError {
            error: format!("{:?}", err)
        }
    }
}
