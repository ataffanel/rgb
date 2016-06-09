// Gameboy video implementation

enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

pub struct Video {
    mode: Mode,
    next_event: usize,

    // Video memories
    vram: Vec<u8>,
    oam: Vec<u8>,

    // Internal representation of the drawn screen
    // Even though pixels only have 2 bits depth, we represent them with 1 byte for easiness
    screen: Vec<u8>,
}

impl Video {
    pub fn new() -> Video {
        Video {
            mode: Mode::Mode1,
            next_event: 0,
            vram: vec![0; 8*1024],
            oam: vec![0, 160],
            screen: vec![0;166*144],
        }
    }

    pub fn step(&mut self, cycle: usize) {
        if self.next_event >= cycle {

        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            _ if address >= 0x8000 && address < 0xA000 => self.vram[(address&0x1fff) as usize],
            _ => panic!("Address decoding bug: ${:04x} is not in video space.", address),
        }
    }

    pub fn write(&mut self, address:u16, data: u8) {
        match address {
            _ if address >= 0x8000 && address < 0xA000 => self.vram[(address&0x1fff) as usize] = data,
            _ => panic!("Address decoding bug: ${:04x} is not in video space.", address)
        }
    }
}
