
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fmt;

pub struct Cart {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,

    // Cart config
    has_mapper: bool,
    mapper_type: Type,
    ram_size: usize,
    type_str: &'static str,
    ram_data_mask: u8,
    ram_addr_mask: u16,

    // Cart runtime state
    ram_enable: bool,
    ram_banking_mode: bool,
    rom_bank: usize,
    ram_bank: usize,
}

#[derive(Debug)]
pub struct CartLoadError {
    pub error: String,
}

pub enum Type {
    ROM,
    MBC1,
    MBC2,
    MMM01,
    MBC3,
    MBC4,
    MBC5,
    CAMERA,
    TAMA5,
    HUC3,
    HUC1,
    Unknown,
}

impl Cart {
    pub fn load(rom_path: &String, ram_save: Option<&String>) -> Result<Cart, CartLoadError> {
        let mut f = r#try!(File::open(rom_path));
        let mut buffer = Vec::new();
        r#try!(f.read_to_end(&mut buffer));

        let ram_buffer;
        if let Some(ram_filename) = ram_save {
            let mut fr = r#try!(File::open(ram_filename));
            let mut buffer = Vec::new();
            r#try!(fr.read_to_end(&mut buffer));
            ram_buffer = Some(buffer);
        } else {
            ram_buffer = None;
        }

        Cart::init(buffer, ram_buffer)
    }

    pub fn create_from_slice(slice: &[u8]) -> Cart {
        Cart::init(slice.to_vec(), None).unwrap()
    }

    pub fn read(&self, address:u16) -> u8 {
        let bank_offset = self.rom_bank*0x4000;
        let ram_offset = self.ram_bank*0x2000;
        match address {
            _ if address < 0x4000 => if (address as usize) < self.rom.len() { self.rom[address as usize] } else { 0xff },
            _ if address < 0x8000 => self.rom[bank_offset + ((address&0x3fff) as usize)],
            _ if address >= 0xA000 && address < 0xC000 => {
                    if self.ram_size != 0 {self.ram[ram_offset + ((address&0x1fff) as usize)]} else {0}
                }
            _ => { println!("Warning: Reading outside the rom!"); 0 }
        }
    }

    // Implements MBC1 mapper only for now
    pub fn write(&mut self, address:u16, data:u8) {


        match address {
            _ if address < 0x8000 => self.write_mbc(address, data),
            _ if address >= 0xA000 && address < 0xC000 => self.write_ram(address, data),
            _ => { println!("Writing in cart addr {:04x} data {:02x}", address, data); },
        }
    }

    fn write_mbc(&mut self, address: u16, data: u8) {

        match self.mapper_type {
            Type::ROM => (),
            Type::MBC1 => {
                match address & 0x6000 {
                    0x0000 => self.ram_enable = if data&0x0f == 0x0a {true} else {false},
                    0x2000 => {
                        self.rom_bank = (if data==0 {1} else {self.rom_bank} & 0x60) | (data&0x1f) as usize
                    }
                    0x4000 => {
                        if self.ram_banking_mode {
                            self.ram_bank = (data & 0x03) as usize;
                        } else {
                            self.rom_bank = (self.rom_bank & 0x1f) | ((data & 0x03)<<5) as usize;
                        }
                    }
                    0x6000 => {
                        if (data & 0x01) == 0 {
                            self.ram_banking_mode = false;
                            self.rom_bank = (self.rom_bank & 0x1f) | (self.ram_bank << 5);
                            self.ram_bank = 0;
                        } else {
                            self.ram_banking_mode = true;
                            self.ram_bank = (self.rom_bank & 0x60) >> 5;
                            self.rom_bank = self.rom_bank & 0x1f;
                        }
                    },
                    _ => (),
                }
            }
            Type::MBC2 => {
                self.ram_enable = true;
                match address & 0x6000 {
                    0x2000 => self.rom_bank = (data & 0x0F) as usize,
                    _ => (),
                }
            }
            Type::MBC5 => {
                match address & 0x7000 {
                    0x0000...0x1000 => self.ram_enable = if data&0x0f == 0x0a {true} else {false},
                    0x2000 => self.rom_bank = (self.rom_bank & 0x100) | data as usize,
                    0x3000 => self.rom_bank = (self.rom_bank & 0x0FF) | (((data & 0x01) as usize) << 8),
                    0x4000...0x5000 => self.ram_bank = (data & 0x0f) as usize,
                    _ => (),
                }
            }
            _ => panic!("Cart mapper type not supported: {}", self.type_str),
        }
    }

    fn write_ram(&mut self, address: u16, data: u8) {
        let ram_offset = self.ram_bank * 0x2000;

        if self.ram_size != 0 && self.ram_enable {
            self.ram[ram_offset + (((address&self.ram_addr_mask)&0x1fff) as usize)] = data & self.ram_data_mask;
        }
    }

    // Private functions
    fn init(buffer: Vec<u8>, ram_buffer: Option<Vec<u8>>) -> Result<Cart, CartLoadError> {

        // (mbc, has_ram, has_battery, has_timer, has_rumble, type_str)
        let decoded_type = match buffer[0x147] {
            0x00 => (Type::ROM,    false, false, false, false, "ROM ONLY"),
            0x01 => (Type::MBC1,   false, false, false, false, "MBC1"),
            0x02 => (Type::MBC1,   true , false, false, false, "MBC1+RAM"),
            0x03 => (Type::MBC1,   true , true , false, false, "MBC1+RAM+BATTERY"),
            0x05 => (Type::MBC2,   false, false, false, false, "MBC2"),
            0x06 => (Type::MBC2,   false, true , false, false, "MBC2+BATTERY"),
            0x08 => (Type::ROM,    true , false, false, false, "ROM+RAM"),
            0x09 => (Type::ROM,    true , true , false, false, "ROM+RAM+BATTERY"),
            0x0B => (Type::MMM01,  false, false, false, false, "MMM01"),
            0x0C => (Type::MMM01,  true , false, false, false, "MMM01+RAM"),
            0x0D => (Type::MMM01,  true , true , false, false, "MMM01+RAM+BATTERY"),
            0x0F => (Type::MBC3,   false, true , true , false, "MBC3+TIMER+BATTERY"),
            0x10 => (Type::MBC3,   false, true , true , false, "MBC3+TIMER+RAM+BATTERY"),
            0x11 => (Type::MBC3,   false, false, false, false, "MBC3"),
            0x12 => (Type::MBC3,   true , false, false, false, "MBC3+RAM"),
            0x13 => (Type::MBC3,   true , true , false, false, "MBC3+RAM+BATTERY"),
            0x15 => (Type::MBC4,   false, false, false, false, "MBC4"),
            0x16 => (Type::MBC4,   true , false, false, false, "MBC4+RAM"),
            0x17 => (Type::MBC4,   true , true , false, false, "MBC4+RAM+BATTERY"),
            0x19 => (Type::MBC5,   false, false, false, false, "MBC5"),
            0x1A => (Type::MBC5,   true , false, false, false, "MBC5+RAM"),
            0x1B => (Type::MBC5,   true , true , false, false, "MBC5+RAM+BATTERY"),
            0x1C => (Type::MBC5,   false, false, false, true , "MBC5+RUMBLE"),
            0x1D => (Type::MBC5,   true , false, false, true , "MBC5+RUMBLE+RAM"),
            0x1E => (Type::MBC5,   true , true , false, true , "MBC5+RUMBLE+RAM+BATTERY"),
            0xFC => (Type::CAMERA, false, false, false, false, "POCKET CAMERA"),
            0xFD => (Type::TAMA5,  false, false, false, false, "BANDAI TAMA5"),
            0xFE => (Type::HUC3,   false, false, false, false, "HuC3"),
            0xFF => (Type::HUC1,   true , true , false, false, "HuC1+RAM+BATTERY"),
            _    => (Type::Unknown,false, false, false, false, "Uknown"),
        };

        let has_mapper = if let Type::ROM = decoded_type.0 { false } else { true };
        let has_ram = decoded_type.1;

        let ram;
        let ram_size;
        let ram_data_mask;
        let ram_addr_mask;
        if has_ram {
            ram_size = match buffer[0x149] {
                1 => 2*1024,
                2 => 8*1024,
                3 => 32*1024,
                _ => 0,
            };
            ram_data_mask = 0xff;
            ram_addr_mask = (ram_size - 1) as u16;
        } else if let Type::MBC2 = decoded_type.0 {
            ram_size = 512;
            ram_addr_mask = 0x01ff;
            ram_data_mask = 0x0F;
        } else {
            ram_size = 0;
            ram_addr_mask = 0x0;
            ram_data_mask = 0x0;
        }

        if let Some(ram_buffer) = ram_buffer {
            if ram_buffer.len() != ram_size {
                return Err(CartLoadError::from("Ram save file has wrong size."));
            }
            ram = ram_buffer;
        } else {
            ram = vec![0;ram_size];
        }

        Ok(Cart {
            rom: buffer,
            ram: ram,

            has_mapper: has_mapper,
            mapper_type: decoded_type.0,
            ram_banking_mode: false,
            ram_enable: false,
            rom_bank: 1,
            ram_bank: 0,

            ram_size: ram_size,
            ram_data_mask: ram_data_mask,
            ram_addr_mask: ram_addr_mask,

            type_str: decoded_type.5,
        })
    }

}

impl fmt::Display for Cart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cartrige type: {}\n\
                   Rom size: {}\n\
                   Ram size: {}", self.type_str, self.rom.len(), self.ram_size)
    }
}


impl From<io::Error> for CartLoadError {
    fn from(err: io::Error) -> CartLoadError {
        CartLoadError {
            error: format!("{:?}", err)
        }
    }
}

impl From<&'static str> for CartLoadError {
    fn from(err: &'static str) -> CartLoadError {
        CartLoadError {
            error: format!("{:?}", err)
        }
    }
}
