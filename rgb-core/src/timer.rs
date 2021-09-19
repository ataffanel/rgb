use crate::cpu;

pub struct Timer {
    div_full: u16,
    reg_tima: u8,
    reg_tma: u8,
    reg_tac: u8,

    prev_cycle: usize,
    prev_timer_inc: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div_full: 0,
            reg_tima: 0,
            reg_tma: 0,
            reg_tac: 0,

            prev_cycle: 0,
            prev_timer_inc: false,
        }
    }

    // Memory access
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xff04 => (self.div_full>>8) as u8,
            0xff05 => self.reg_tima,
            0xff06 => self.reg_tma,
            0xff07 => self.reg_tac,
            _ => panic!("Read timer address decoding bug"),
        }
    }

    pub fn write(&mut self, address: u16, data: u8) {
        match address {
            0xff04 => self.div_full = 0,
            0xff05 => self.reg_tima = data,
            0xff06 => self.reg_tma = data,
            0xff07 => self.reg_tac = data,
            _ => panic!("Write timer address decoding bug"),
        };
    }

    // Run!
    pub fn step(&mut self, cycle: usize) -> u8 {
        let mut irq = 0;
        let step = cycle.wrapping_sub(self.prev_cycle) as u16;
        self.prev_cycle = cycle;

        for _ in 0..step {
            self.div_full = self.div_full.wrapping_add(1);

            let timer_inc = match self.reg_tac&0x03 {
                0 => self.div_full&(1<<9) != 0,
                1 => self.div_full&(1<<3) != 0,
                2 => self.div_full&(1<<5) != 0,
                3 => self.div_full&(1<<7) != 0,
                _ => panic!("Timer bug"),
            };

            if self.reg_tac&0x04 != 0 && self.prev_timer_inc && !timer_inc {
                self.reg_tima = self.reg_tima.wrapping_add(1);
                if self.reg_tima == 0 {
                    self.reg_tima = self.reg_tma;
                    irq = cpu::IRQ_TIMER;
                }
            }
            self.prev_timer_inc = timer_inc;
        }

        irq
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
