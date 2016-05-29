
use std::io;
use std::io::prelude::*;
use std::fs::File;

use std::fmt;

pub struct Cart {
    pub rom: Vec<u8>,
}

pub struct CartLoadError {
    pub error: String,
}

impl Cart {
    pub fn load(rom_path: &String) -> Result<Cart, CartLoadError> {
        let mut f = try!(File::open(rom_path));
        let mut buffer = Vec::new();

        try!(f.read_to_end(&mut buffer));

        Ok(Cart {
            rom: buffer,
        })
    }

    // Simple version without mapping. Needs to be enhanced for more complex roms
    pub fn read(&self, address:u16) -> u8 {
        match address {
            _ if address < 0x8000 => if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0 },
            _ => { println!("Warning: Reading outside the rom!"); 0 }
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

impl fmt::Debug for CartLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
