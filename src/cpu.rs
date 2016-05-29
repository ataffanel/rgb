// Emulation of the Game Boy LR35902 cpu

use cart;

const ZERO_FLAG: u8 = 1<<7;
const SUBSTRACT_FLAG: u8 = 1<<6;
const HALF_CARRY_FLAG: u8 = 1<<5;
const CARRY_FLAG: u8 = 1<<4;

struct Regs {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,

    pc: u16,
    lr: u16,
}

pub struct Cpu {
    regs: Regs,
    mem: cart::Cart,
    cycle: usize,
}

impl Cpu {
    pub fn new(cart: cart::Cart) -> Cpu {
        Cpu {
            regs: Regs {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                f: 0,
                h: 0,
                l: 0,

                pc: 0,
                lr: 0,
            },
            mem: cart,
            cycle: 0,
        }
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.regs.pc = pc;
    }

    pub fn execute_until(&mut self, cycle: usize) {
        while self.cycle < cycle {
            self.cycle += self.decode();
        }
    }

    pub fn get_cycle(&self) -> usize { self.cycle }

    // Pivate methods

    fn decode(&mut self) -> usize {
        let instr = self.mem.read(self.regs.pc);
        match instr {
            0x00 => self.nop(),
            _ => panic!("Uknown instruction op: {:02x} at 0x{:04x}!", instr, self.regs.pc)
        }
    }

    fn nop(&mut self) -> usize {
        println!("{:04x}: nop", self.regs.pc);
        self.regs.pc += 1;
        4
    }
}
