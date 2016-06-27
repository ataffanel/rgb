// Implements the memory multiplexer, the fast ram and the work ram.
// This is a gameboy for now, not a gameboy color, so no banking of the work ram

use cart::Cart;
use video::Video;
use bootstrap::Bootstrap;
use joypad::Joypad;

pub struct Mem {
    bootstrap: Bootstrap,
    cart: Cart,
    pub work: Vec<u8>,
    pub hram: Vec<u8>,
    page0_mode: u8,
    pub reg_ie: u8,
    pub reg_if: u8,

    pub video: Video,
    pub joypad: Joypad,
}

impl Mem {
    pub fn new(bootstrap: Bootstrap, cart: Cart) -> Mem {
        Mem {
            bootstrap: bootstrap,
            cart: cart,
            work: vec![0; 8*1024],
            hram: vec![0; 256],
            reg_ie: 0,
            reg_if: 0,
            page0_mode: 0,
            video: Video::new(),
            joypad: Joypad::new(),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            _ if address < 0x0100 && self.page0_mode == 0 => self.bootstrap.read(address),
            _ if address < 0x8000 => self.cart.read(address), // Cart ROM
            _ if address < 0xA000 => self.video.read(address), // VRAM
            _ if address < 0xC000 => self.cart.read(address), // Cart RAM
            _ if address < 0xFE00 => self.work[(address&0x1FFF) as usize],
            _ if address < 0xFEA0 => self.video.read(address), // OAM
            _ if address < 0xFF00 => 0, // Not usable, ignored
            _ if address < 0xFF80 => match address {
                0xFF00 => self.joypad.read(address),
                0xFF0F => self.reg_if,
                0xff50 => self.page0_mode,
                _ if address & 0x00f0 == 0x40 => self.video.read(address),
                _ => 0xff,
            }, // IO registers
            _ if address < 0xFFFF => self.hram[(address&0xFF) as usize],
            0xffff => self.reg_ie,
            _ => panic!("Read address decoding bug"),
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
                0xFF00 => self.joypad.write(address, data),
                0xFF01 => print!("\x1b[1;34m{}\x1b[0m", (data as char).to_string()),
                0xFF0F => self.reg_if = data,
                0xff50 if self.page0_mode == 0 => self.page0_mode = data,
                _ if address & 0x00f0 == 0x40 => self.video.write(address, data),
                _ => (),
            }, // IO registers
            _ if address < 0xFFFF => self.hram[(address&0xFF) as usize] = data,
            0xffff => self.reg_ie = data,
            _ => panic!("Write address decoding bug"),
        }
    }
}
