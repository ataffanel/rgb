// Emulation of the Game Boy LR35902 cpu

use cart;

use std::fmt;

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

impl fmt::Debug for Regs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug = String::new();
        debug.push_str(&format!("+-Registers--\n",));
        debug.push_str(&format!("| A: {:02x} F: {:02x}\n", self.a, self.f));
        debug.push_str(&format!("| B: {:02x} C: {:02x}\n", self.b, self.c));
        debug.push_str(&format!("| D: {:02x} E: {:02x}\n", self.d, self.e));
        debug.push_str(&format!("| H: {:02x} L: {:02x}\n", self.h, self.l));
        debug.push_str(&format!("| SP: {:02x}\n", self.sp));
        debug.push_str(&format!("| PC: {:02x}\n", self.pc));
        write!(f, "{}", debug)
    }
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

    fn get_flag(&mut self, flag: u8) -> bool { (self.regs.f & flag) != 0 }

    fn decode(&mut self) -> usize {
        let instr = self.mem.read(self.regs.pc);
        match instr {
            0x00 => self.nop(),
            0x20 => self.jr_nz_r8(),
            0x01 | 0x11 | 0x21 | 0x31 => self.ld16_val(instr),
            0x02 | 0x12 | 0x22 | 0x32 => self.store8_ind(instr),
            0xA8 ... 0xAD | 0xAF => self.xor_reg(instr),
            0xCB => self.decode_cb(),
            _ => {
                println!("\n{:?}", self.regs);
                panic!("Uknown instruction op: 0x{:02x} at addr 0x{:04x}!", instr, self.regs.pc)
            }
        }
    }

    fn nop(&mut self) -> usize {
        println!("{:04x}: NOP", self.regs.pc);
        self.regs.pc += 1;
        4
    }

    fn jr_nz_r8(&mut self) -> usize {
        let val = self.mem.read(self.regs.pc+1) as i8;
        println!("{:04x}: JR NZ, {}", self.regs.pc, val);
        if !self.get_flag(ZERO_FLAG) {
            let newpc = (self.regs.pc as i32) + 2 + (val as i32);
            self.regs.pc = newpc as u16;
            12
        } else {
            self.regs.pc += 2;
            8
        }
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

    fn decode_cb(&mut self) -> usize {
        self.regs.pc += 1;
        let instr = self.mem.read(self.regs.pc);

        4 + match instr {
            _ if (instr & 0xC0) == 0x40 => self.bit((instr >> 3) & 0x07, instr & 0x07),
            _ => {
                println!("\n{:?}", self.regs);
                panic!("Uknown prefixed instruction op: 0x{:02x} at addr 0x{:04x}!", instr, self.regs.pc)
            }
        }
    }

    fn bit(&mut self, bit: u8, reg_id: u8) -> usize {
        self.set_flag(HALF_CARRY_FLAG, true);
        self.set_flag(SUBSTRACT_FLAG, false);

        let (flag, cycles, reg_name) = match reg_id {
            // Register versions
            0 => (self.regs.b & (1<<bit) == 0, 8, "B"),
            1 => (self.regs.c & (1<<bit) == 0, 8, "C"),
            2 => (self.regs.d & (1<<bit) == 0, 8, "D"),
            3 => (self.regs.e & (1<<bit) == 0, 8, "E"),
            4 => (self.regs.h & (1<<bit) == 0, 8, "H"),
            5 => (self.regs.l & (1<<bit) == 0, 8, "L"),
            7 => (self.regs.a & (1<<bit) == 0, 8, "A"),
            // Indirect HL version
            6 => {
                let hl = (self.regs.h as u16) << 8 | self.regs.l as u16;
                (self.mem.read(hl) & (1<<bit) == 0, 16, "B")
            },

            _ => panic!("Bug in decoding")
        };

        self.set_flag(ZERO_FLAG, flag);

        println!("{:04x}: BIT {}, {}", self.regs.pc-1, bit, reg_name);
        self.regs.pc += 1;
        4
    }
}
