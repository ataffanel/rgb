// Gameboy video implementation

use cpu;

enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

#[derive(Copy,Clone,PartialEq)]
enum Palette {
    BLANK,
    BGP,
    OBP0,
    OBP1,
}

#[derive(Copy,Clone)]
struct Pixel {
    color: usize,
    palette: Palette,
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

                        if self.registers[STAT] & (1<<5) != 0 {
                            irq |= cpu::IRQ_LCDSTAT;
                        }

                        MODE2_CLK
                    } else { // Switch to VBLANK
                        self.mode = Mode::Mode1;
                        self.registers[STAT] |= 0x01;
                        self.image_ready = true;

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

                        if self.registers[STAT] & (1<<5) != 0 {
                            irq |= cpu::IRQ_LCDSTAT;
                        }

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

                    if self.registers[STAT] & (1<<3) != 0 {
                        irq |= cpu::IRQ_LCDSTAT;
                    }

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
            _ if address & 0x00f0 == 0x40 => self.registers[(address&0x000f) as usize] = data,
            _ => panic!("Address decoding bug: ${:04x} is not in video space.", address)
        }
    }

    // Render the LY line in the internal screen buffer
    fn render_line(&mut self) {
        let current_line = self.registers[LY] as usize;
        if current_line>143 { panic!("render_line should not be called during VBLANK!"); }
        let mut line_pixels  = [Pixel { color:0, palette:Palette::BLANK }; LINE_WIDTH];

        if self.registers[LCDC]&(1<<0) != 0 {
            self.draw_background(&mut line_pixels);
        }
        if self.registers[LCDC]&(1<<5) != 0 {
            self.draw_window(&mut line_pixels);
        }
        if self.registers[LCDC]&(1<<1) != 0 {
            self.draw_sprites(&mut line_pixels);
        }

        let mut line = &mut self.screen[current_line*LINE_WIDTH*3 .. (current_line+1)*LINE_WIDTH*3];
        let mut i = 0;
        for pixel in line_pixels.into_iter() {
            let color = match pixel.palette {
                Palette::BLANK => 0,
                Palette::BGP => ((self.registers[BGP] as usize) >> (pixel.color*2))&0x03,
                Palette::OBP0 => ((self.registers[OBP0] as usize) >> (pixel.color*2))&0x03,
                Palette::OBP1 => ((self.registers[OBP1] as usize) >> (pixel.color*2))&0x03,
            };

            line[(3*i)+0]= self.color_map[color][2];
            line[(3*i)+1]= self.color_map[color][1];
            line[(3*i)+2]= self.color_map[color][0];
            i += 1;
        }
    }

    fn draw_background(&mut self, line_pixels: &mut [Pixel]) {
        let current_line = self.registers[LY] as usize;
        let bg_x = self.registers[SCX] as usize;
        let bg_y = (current_line+(self.registers[SCY] as usize))&0xff;
        let y_in_tile = bg_y&0x07;
        for i in 0..LINE_WIDTH {
            let x = (bg_x + i)&0xff;
            let tile_pos = ((bg_y&0xF8)<<2) | ((x>>3)&0x1F);
            let x_in_tile = x&0x07;

            line_pixels[i].color = self.get_bg_tile_pixel(tile_pos, y_in_tile, x_in_tile);
            line_pixels[i].palette = Palette::BGP;
        }
    }

    fn draw_window(&mut self, line_pixels: &mut [Pixel]) {
        let current_line = self.registers[LY] as isize;
        let x_in_window = (self.registers[WX] as isize) - 7;
        let y_in_window = current_line - (self.registers[WY] as isize);

        if y_in_window >= 0 && y_in_window < 144 {
            for x in 0..LINE_WIDTH {
                let x_in_window = x_in_window + (x as isize);
                if x_in_window >= 0 && x_in_window < (LINE_WIDTH as isize) {
                    let (wx, wy) = (x_in_window as usize, y_in_window as usize);
                    let tile_pos = ((wy&0xF8)<<2) | ((wx>>3)&0x1F);
                    let tile_col = wx & 0x07;
                    let tile_line = wy & 0x07;

                    let tile_map_addr: usize = if self.registers[LCDC]&(1<<6)==0 {0x1800} else {0x1c00};
                    let tile_id = self.vram[tile_map_addr + tile_pos] as u8;

                    line_pixels[x].palette = Palette::BGP;
                    line_pixels[x].color = Video::get_tile_color(&self.vram, self.registers[LCDC]&(1<<6)!=0,
                                                                tile_id, tile_col, tile_line);
                }
            }
        }
    }

    fn get_bg_tile_pixel(&mut self, tile_pos: usize, line: usize, col: usize) -> usize {
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

        //((self.registers[BGP] as usize) >> (unmapped*2))&0x03
        unmapped
    }

    fn get_tile_color(vram: &[u8], signed_id: bool, tile_id: u8, col: usize, line: usize) -> usize {
        let tile_address;

        if signed_id {
            tile_address = (0x1000 + (((tile_id as i8) as isize)*16)) as usize;
        } else {
            tile_address = 0x0000 + ((tile_id as usize)*16);
        }

        let low = vram[tile_address+(2*line)] as usize;
        let high = vram[tile_address+(2*line)+1] as usize;

        (((high>>(7-col&0x07))&0x01)<<1) | ((low>>(7-col&0x07))&0x01)
    }

    fn draw_sprites(&mut self, line_pixels: &mut [Pixel]) {
        let ly = self.registers[LY] as usize;
        let height = if self.registers[LCDC]&(1<<2) == 0 { 8 } else { 16 };

        for i in 0..40 {
            let attribute = &self.oam[i*4..(i+1)*4];
            if (ly as u8) >= attribute[0].wrapping_sub(16) && (ly as u8) < attribute[0].wrapping_sub(16-height) {
                // Decoding sprite attribute
                let above_bg = attribute[3]&0x80 == 0;
                let pallette = if attribute[3]&0x10 == 0 { Palette::OBP0 } else { Palette::OBP1 };
                let x_flip = attribute[3]&(1<<5) != 0;
                let y_flip = attribute[3]&(1<<6) != 0;

                // Plotting sprite
                let (start, col) = if attribute[1]<8 { (0, 8-attribute[1]) } else { (attribute[1].wrapping_sub(8), 0) };
                let mut col = col as usize;
                for x in start..attribute[1] {
                    let x = x as usize;
                    let start = start as usize;
                    if x<LINE_WIDTH {
                        if line_pixels[x].palette == Palette::BGP && above_bg || line_pixels[x].color == 0 {
                            let mut line = ly-(attribute[0].wrapping_sub(16) as usize);
                            let mut rcol = col;
                            if x_flip { rcol = 7 - col; }
                            if y_flip { line = height as usize - 1 - line; }
                            let color = Video::get_tile_color(&self.vram, false, attribute[2], rcol, line);
                            if color != 0 {
                                line_pixels[x].color = color;
                                line_pixels[x].palette = pallette;
                            }
                            col += 1;
                        }
                    }
                }
            }
        }
    }
}
