// Implements the memory multiplexer, the fast ram and the work ram.
// This is a gameboy for now, not a gameboy color, so no banking of the work ram

use cart::Cart;
use video::Video;

pub struct Mem {
    cart: Cart,
    work: Vec<u8>,
    hram: Vec<u8>,
    interrupts: u8,

    pub video: Video,
}

impl Mem {
    pub fn new(cart: Cart) -> Mem {
        Mem {
            cart: cart,
            work: vec![0; 8*1024],
            hram: vec![0; 256],
            interrupts: 0,
            video: Video::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            _ if address < 0x8000 => self.cart.read(address), // Cart ROM
            _ if address < 0xA000 => self.video.read(address), // VRAM
            _ if address < 0xC000 => self.cart.read(address), // Cart RAM
            _ if address < 0xFE00 => self.work[(address&0x1FFF) as usize],
            _ if address < 0xFEA0 => self.video.read(address), // OAM
            _ if address < 0xFF00 => 0, // Not usable, ignored
            _ if address < 0xFF80 => match address {
                _ if address & 0x00f0 == 0x40 => self.video.read(address),
                _ => 0,
            }, // IO registers
            _ if address < 0xFFFF => self.hram[(address&0xFF) as usize],
            _ => self.interrupts, //0xFFFF !
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            _ if address < 0x8000 => self.cart.write(address, data), // Cart ROM
            _ if address < 0xA000 => self.video.write(address, data), // VRAM
            _ if address < 0xC000 => self.cart.write(address, data), // Cart RAM
            _ if address < 0xFE00 => self.work[(address&0x1FFF) as usize] = data,
            _ if address < 0xFEA0 => self.video.write(address, data), // OAM
            _ if address < 0xFF00 => (), // Not usable, ignored
            _ if address < 0xFF80 => match address {
                _ if address & 0x00f0 == 0x40 => self.video.write(address, data),
                _ => (),
            }, // IO registers
            _ if address < 0xFFFF => self.hram[(address&0xFF) as usize] = data,
            _ => self.interrupts = data, //0xFFFF !
        }
    }
}
