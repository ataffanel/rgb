// Gameboy video implementation

use cpu;

enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

pub struct Video {
    mode: Mode,
    next_event: usize,
    enabled: bool,

    // Video memories
    pub vram: Vec<u8>,
    oam: Vec<u8>,
    registers: Vec<u8>,

    // Internal representation of the drawn screen
    // Even though pixels only have 2 bits depth, we represent them in full color 3bytes
    pub screen: Vec<u8>,
    pub image_ready: bool,
    color_map: &'static [ [u8; 3]; 4 ],
}

// Cofiguration register address in the internal video register memory
// (Starts at 0xFF40 in the GB memory map)
const LCDC: usize = 0x00;
const STAT: usize = 0x01;
const SCY: usize = 0x02;
const SCX: usize = 0x03;
const LY: usize = 0x04;
const LYC: usize = 0x05;
const DMA: usize = 0x06;
const BGP: usize = 0x07;
const OBP0: usize = 0x08;
const OBP1: usize = 0x09;
const WY: usize = 0x0A;
const WX: usize = 0x0B;


const MODE0_CLK: usize = 204;
const MODE1_LINE_CLK: usize = 456;
const MODE2_CLK: usize = 80;
const MODE3_CLK: usize = 172;

enum Interrupt {
    VBlank,

}

// const COLOR_MAPPING: &'static [ [u8; 3]; 4 ] = &[[223, 249, 206],
//                                                  [127,127,127],
//                                                  [64,64,64],
//                                                  [0,0,0],];
const COLOR_MAPPING: &'static [ [u8; 3]; 4 ] = &[[149, 176, 29],
                                                 [108, 136, 41],
                                                 [58, 100, 60],
                                                 [29, 62, 30],];

const LINE_WIDTH: usize = 160;

fn get_shade(palette: u8, color: u8) -> u8 {
    (palette >> (color*2)) & 0x03
}

impl Video {
    pub fn new() -> Video {
        Video {
            mode: Mode::Mode1,
            next_event: 0,
            enabled: false,
            vram: vec![0; 8*1024],
            oam: vec![0; 160],
            registers: vec![0; 16],
            screen: vec![0;LINE_WIDTH*144*3],
            image_ready: false,
            color_map: COLOR_MAPPING,
        }
    }

    pub fn step(&mut self, cycle: usize) -> u8 {
        let mut irq = 0;

        self.image_ready = false;

        if cycle >= self.next_event {
            self.registers[STAT] &= 0xfc;

            self.next_event += match self.mode {
                Mode::Mode0 => {
                    self.registers[LY] += 1;

                    if self.registers[LY] < 144 {
                        self.mode = Mode::Mode2;
                        self.registers[STAT] |= 0x02;

                        MODE2_CLK
                    } else { // Switch to VBLANK
                        self.mode = Mode::Mode1;
                        self.registers[STAT] |= 0x01;
                        self.image_ready = true;

                        //println!("SCY: {}", self.registers[SCY]);

                        irq |= cpu::IRQ_VBLANK;
                        if self.registers[STAT] & (1<<4) != 0 {
                            irq |= cpu::IRQ_LCDSTAT;
                        }
                        MODE1_LINE_CLK
                    }
                },
                Mode::Mode1 => {
                    self.registers[LY] += 1;

                    if self.registers[LY] > 153 {
                        self.registers[LY] = 0;
                        self.mode = Mode::Mode2;
                        self.registers[STAT] |= 0x02;

                        MODE2_CLK
                    } else {
                        self.registers[STAT] |= 0x01;
                        MODE1_LINE_CLK
                    }
                },
                Mode::Mode2 => {
                    self.mode = Mode::Mode3;
                    self.registers[STAT] |= 0x03;
                    MODE3_CLK
                },
                Mode::Mode3 => {
                    self.render_line();
                    self.mode = Mode::Mode0;
                    self.registers[STAT] |= 0x00;
                    MODE0_CLK
                },
            };
        };

        if self.registers[STAT] & (1<<6) != 0 && self.registers[LYC] == self.registers[LY] {
            irq |= cpu::IRQ_LCDSTAT;
        }
        irq
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            _ if address >= 0x8000 && address < 0xA000 => match self.mode {
                Mode::Mode3 => 0xff,
                _ => self.vram[(address&0x1fff) as usize],
            },
            _ if address >= 0xFE00 && address <= 0xFE9F => match self.mode {
                Mode::Mode2 | Mode::Mode3 => 0xff,
                _ => self.oam[(address & 0xff) as usize],
            },
            _ if address & 0x00f0 == 0x40 => self.registers[(address&0x000f) as usize],
            _ => panic!("Address decoding bug: ${:04x} is not in video space.", address),
        }
    }

    pub fn write(&mut self, address:u16, data: u8) {
        match address {
            _ if address >= 0x8000 && address < 0xA000 => match self.mode {
                //Mode::Mode3 => (),
                _ => self.vram[(address&0x1fff) as usize] = data,
            },
            _ if address >= 0xFE00 && address <= 0xFE9F => match self.mode {
                //Mode::Mode2 | Mode::Mode3 => (),
                _ => self.oam[(address & 0xff) as usize] = data,
            },
            //0xff46 => panic!("This is DMA :-("),
            _ if address & 0x00f0 == 0x40 => self.registers[(address&0x000f) as usize] = data,
            _ => panic!("Address decoding bug: ${:04x} is not in video space.", address)
        }
    }

    // Render the LY line in the internal screen buffer
    fn render_line(&mut self) {
        let current_line = self.registers[LY] as usize;
        if current_line>143 { panic!("render_line should not be called during VBLANK!"); }

        // Drawing background
        //let bg_pos = ((current_line+(self.registers[SCY] as usize))*256) + self.registers[SCX] as usize;
        let bg_x = self.registers[SCX] as usize;
        let bg_y = (current_line+(self.registers[SCY] as usize))&0xff;
        let y_in_tile = bg_y&0x07;
        for i in 0..LINE_WIDTH {
            let x = (bg_x + i)&0xff;
            let tile_pos = ((bg_y&0xF8)<<2) | ((x>>3)&0x1F);
            let x_in_tile = x&0x07;

            let color = self.get_tile_pixel(tile_pos, y_in_tile, x_in_tile);

            let mut line = &mut self.screen[current_line*LINE_WIDTH*3 .. (current_line+1)*LINE_WIDTH*3];
            let pixel = &mut line[i*3 .. (i+1)*3];
            pixel[0] = self.color_map[color][2];
            pixel[1] = self.color_map[color][1];
            pixel[2] = self.color_map[color][0];
        }
    }

    fn get_tile_pixel(&mut self, tile_pos: usize, line: usize, col: usize) -> usize {
        let tile_map_addr: usize = if self.registers[LCDC]&0x08==0 {0x1800} else {0x1c00};
        let tile_id = self.vram[tile_map_addr + tile_pos] as u8;
        let tile_address;

        if self.registers[LCDC]&0x10 == 0 {
            tile_address = (0x1000 + (((tile_id as i8) as isize)*16)) as usize;
        } else {
            tile_address = 0x0000 + ((tile_id as usize)*16);
        }

        let low = self.vram[tile_address+(2*line)] as usize;
        let high = self.vram[tile_address+(2*line)+1] as usize;

        let unmapped = (((high>>(7-col&0x07))&0x01)<<1) | ((low>>(7-col&0x07))&0x01);

        ((self.registers[BGP] as usize) >> (unmapped*2))&0x03
    }
}
