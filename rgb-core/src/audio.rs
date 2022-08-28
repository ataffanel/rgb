pub struct Audio {
    pub audio_buffer: Vec<i16>,
    registers: [u8; REGISTER_LENGTH],

    sample_timer: f64,
    current_cycle: usize,

    // Sound generators
    square1: SquareGenerator,
    square2: SquareGenerator,
}


const NR10: u16 = 0xff10;

const NR20: u16 = 0xff15;

const NR30: u16 = 0xFF1A;

const _NR40: u16 = 0xFF1F;

const NR52: u16 = 0xff26;

const REGISTER_LENGTH: usize = 0x40;

const SAMPLE_PERIOD: f64 =  (95.0 * 739.2) / 735.0;

impl Audio {
    pub(crate) fn new() -> Audio {
        Audio{
            audio_buffer: Vec::new(),
            registers: [0; REGISTER_LENGTH],
            sample_timer: 0.0,
            current_cycle: 0,
            square1: SquareGenerator::new(true),
            square2: SquareGenerator::new(false),
        }
    }

    pub(crate) fn read(&self, address: u16) -> u8 {
        let address = address as usize - 0xff10;
        assert!(address < REGISTER_LENGTH);
        self.registers[address as usize]
    }

    pub(crate) fn write(&mut self, address: u16, data: u8) {
        match address {
            _ if address < NR20 => self.square1.set_register(address - NR10, data),
            _ if address < NR30 => self.square2.set_register(address - NR20, data),

            NR52 => println!("Write NR52 {}", data),
            _ => (),
        }
    }

    pub(crate) fn step(&mut self, cycle: usize) {

        while self.current_cycle < cycle {
            self.square1.step(self.current_cycle);
            self.square2.step(self.current_cycle);
    
            self.sample_timer += 1.0;
            self.current_cycle += 1;
    
            let sample = self.square1.sample as i16 + self.square2.sample as i16;
    
            while self.sample_timer > SAMPLE_PERIOD {
                self.audio_buffer.push(sample * 255);
                self.sample_timer -= SAMPLE_PERIOD;
            }
        }
        
    }
}

// Sound generators
#[derive(Default, Debug)]
struct SquareGenerator {
    enable: bool,

    frequency: u16,
    has_sweep: bool,

    sweep_period: u8,
    sweep_neg: bool,
    sweep_shift: u8,

    duty: u8,
    length: u8,
    length_enable: bool,

    volume: u8,
    envelope_add: bool,
    envelope_period: u8,
    envelope_timer: u8,

    trigger: bool,

    // Output
    square_step: u8,
    frequency_timer: usize,
    frequency_timer_last_cycle: usize,
    sample: u8,

    // Times
    last_frame_step_cycle: usize,
    frame_counter: u8,
}

impl SquareGenerator {
    fn new(has_sweep: bool) -> Self {
        Self { has_sweep, ..Default::default() }
    }

    fn set_register(&mut self, address: u16, data: u8) {
        match address {
            0 => if self.has_sweep { 
                self.sweep_period = (data & 0x70) >> 4;
                self.sweep_neg = (data & 0x80) != 0;
                self.sweep_shift = data & 0x07;
             },
            1 => {
                self.length = data & 0x3f;
                self.duty = (data & 0xC0) >> 6;
            },
            2 => {
                self.envelope_period = data & 0x07;
                self.envelope_add = (data & 0x08) != 0;
                self.volume = (data & 0xf0) >> 4;
            },
            3 => self.frequency = (self.frequency & 0xff00) | data as u16,
            4 => {
                self.frequency = (self.frequency & 0x00ff) | ((data as u16 & 0x07) << 8);
                self.length_enable = (data & 0x40) != 0;
                self.trigger = (data & 0x80) != 0;
            },
            _ => assert!(false, "Trying to write on unimplemented register"),
        }

        // dbg!(&self);
    }

    fn step(&mut self, cycle: usize) {
        if self.trigger {
            self.enable = true;
            // self.volume = 15;
            self.trigger = false;
        }

        if self.enable {

            // Frame sequencer  
            if self.last_frame_step_cycle + 2048 < cycle {
                // Length
                if self.frame_counter % 2 == 0 {
                    if self.length_enable && self.length > 0 {
                        self.length -= 1;
                        if self.length == 0 {
                            self.enable = false;
                        }
                    }
                }

                // Volume
                if self.frame_counter == 7 {
                    if self.envelope_period != 0 {
                        if self.envelope_timer >= self.envelope_period {
                            self.envelope_timer = 0;
                            // dbg!(self.volume);
                            // assert!(false);
                            if self.envelope_add && self.volume != 15 {
                                self.volume += 1;
                            }
                            if !self.envelope_add && self.volume != 0 {
                                self.volume -= 1;
                            }
                        } else {
                            self.envelope_timer += 1;
                        }
                    }
                }

                // Sweep
                if self.frame_counter == 2 || self.frame_counter == 6 {
                    // Sweep
                }

                self.frame_counter += 1;
                if self.frame_counter > 7 {
                    self.frame_counter = 0;
                }
            }

            // Square generations
            if self.frequency_timer_last_cycle != 0 {
                self.frequency_timer += cycle - self.frequency_timer_last_cycle;
                let period = (2048 - self.frequency as usize) * 4;

                if self.frequency_timer >= period {
                    self.frequency_timer -= period;
                    self.square_step += 1;
                    if self.square_step > 7 {
                        self.square_step = 0;
                    }
                }

                let state = match self.duty {
                    0 => self.square_step >= 1,
                    1 => self.square_step >= 2,
                    2 => self.square_step >= 4,
                    3 => self.square_step >= 6,
                    _ => unimplemented!(),
                };
                self.sample = if state { self.volume } else { 0 };
                // if state {
                //     dbg!(self.sample);
                // }
            }
            
            self.frequency_timer_last_cycle = cycle;
        } else {
            self.sample = 0;
        }

    }

}