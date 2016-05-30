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
    sp: u16,
}

pub struct Cpu {
    regs: Regs,
    mem: cart::Cart,
    cycle: usize,
}

enum Op {
    xor,
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
                sp: 0,
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

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.regs.f |= flag;
        } else {
            self.regs.f &= !flag;
        }
    }

    fn decode(&mut self) -> usize {
        let instr = self.mem.read(self.regs.pc);
        match instr {
            0x00 => self.nop(),
            0x01 | 0x11 | 0x21 | 0x31 => self.ld16_val(instr),
            0x02 | 0x12 | 0x22 | 0x32 => self.store8_ind(instr),
            0xA8 ... 0xAD | 0xAF => self.xor_reg(instr),
            _ => panic!("Uknown instruction op: 0x{:02x} at addr 0x{:04x}!", instr, self.regs.pc)
        }
    }

    fn nop(&mut self) -> usize {
        println!("{:04x}: NOP", self.regs.pc);
        self.regs.pc += 1;
        4
    }

    fn ld16_val(&mut self, instr: u8) -> usize {
        let (val_l, val_h) = (self.mem.read(self.regs.pc+1), self.mem.read(self.regs.pc+2));

        let reg_name = match instr {
            0x01 => {self.regs.b = val_h; self.regs.c = val_l; "BC"},
            0x11 => {self.regs.d = val_h; self.regs.e = val_l; "DE"},
            0x21 => {self.regs.h = val_h; self.regs.l = val_l; "HL"},
            0x31 => {self.regs.sp = (val_h as u16) << 8 | val_l as u16; "SP"},
            _ => panic!("Bug in decoding")
        };

        if val_l == 0 && val_h == 0 {

        }

        println!("{:04x}: LD {}, ${:02x}{:02x}", self.regs.pc, reg_name, val_h, val_l);
        self.regs.pc += 3;
        12
    }

    fn store8_ind(&mut self, instr: u8) -> usize {
        let reg_name = match instr {
            0x02 => {let addr = (self.regs.b as u16) << 8 | self.regs.c as u16; self.mem.write(addr, self.regs.a); "BC"},
            0x12 => {let addr = (self.regs.d as u16) << 8 | self.regs.e as u16; self.mem.write(addr, self.regs.a); "DE"},
            0x22 => {
                let addr = (self.regs.h as u16) << 8 | self.regs.l as u16;
                self.mem.write(addr, self.regs.a);

                let new_hl = addr+1;
                self.regs.h = (new_hl >> 8) as u8;
                self.regs.l = new_hl as u8;

                "HL+"
            },
            0x32 => {
                let addr = (self.regs.h as u16) << 8 | self.regs.l as u16;
                self.mem.write(addr, self.regs.a);

                let new_hl = addr-1;
                self.regs.h = (new_hl >> 8) as u8;
                self.regs.l = new_hl as u8;

                "HL-"
            },
            _ => panic!("Bug in decoding")
        };

        println!("{:04x}: LD ({}), A", self.regs.pc, reg_name);
        self.regs.pc += 1;
        8
    }

    fn xor_reg(&mut self, instr: u8) -> usize {

        let reg_name = match instr {
            0xA8 => {self.regs.a = self.regs.a ^ self.regs.b; "B"},
            0xA9 => {self.regs.a = self.regs.a ^ self.regs.c; "C"},
            0xAA => {self.regs.a = self.regs.a ^ self.regs.d; "D"},
            0xAB => {self.regs.a = self.regs.a ^ self.regs.e; "E"},
            0xAC => {self.regs.a = self.regs.a ^ self.regs.h; "H"},
            0xAD => {self.regs.a = self.regs.a ^ self.regs.l; "L"},
            0xAF => {self.regs.a = self.regs.a ^ self.regs.a; "A"},
            _ => panic!("Bug in decoding")
        };

        if self.regs.a == 0 {
            self.set_flag(ZERO_FLAG, true);
        } else {
            self.set_flag(ZERO_FLAG, false);
        }

        println!("{:04x}: XOR {}", self.regs.pc, reg_name);
        self.regs.pc += 1;
        4
    }
}
