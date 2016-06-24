
use std::io;
use std::io::prelude::*;
use std::fs::File;

pub struct Cart {
    pub rom: Vec<u8>,
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

        Ok(Cart {
            rom: buffer,
        })
    }

    pub fn create_from_slice(slice: &[u8]) -> Cart {
        Cart {
            rom: slice.to_vec(),
        }
    }

    // Simple version without mapping. Needs to be enhanced for more complex roms
    pub fn read(&self, address:u16) -> u8 {
        match address {
            _ if address < 0x8000 => if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0xff },
            _ => 0xff, //{ println!("Warning: Reading outside the rom!"); 0 }
        }
    }

    pub fn write(&mut self, address:u16, data:u8) {
        ; // Ignoring write for a simple ROM, TODO: Remaping for more complex rom!
    }
}


impl From<io::Error> for CartLoadError {
    fn from(err: io::Error) -> CartLoadError {
        CartLoadError {
            error: format!("{:?}", err)
        }
    }
}
