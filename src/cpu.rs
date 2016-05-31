// Emulation of the Game Boy LR35902 cpu

use cart::Cart;
use mem::Mem;

use std::fmt;

const ZERO_FLAG: u8 = 1<<7;
const SUBSTRACT_FLAG: u8 = 1<<6;
const HALF_CARRY_FLAG: u8 = 1<<5;
const CARRY_FLAG: u8 = 1<<4;

// 8Bit register id as encoded in instructions
const B_REGID: u8 = 0;
const C_REGID: u8 = 1;
const D_REGID: u8 = 2;
const E_REGID: u8 = 3;
const H_REGID: u8 = 4;
const L_REGID: u8 = 5;
const IND_HL_REGID: u8 = 6;
const A_REGID: u8 = 7;

const REG_NAMES: &'static [ &'static str ] = &["B", "C", "D", "E", "H", "L", "(HL)", "A"];

// 16Bit register id as encoded in instructions
const BC_REGID: u8 = 0;
const DE_REGID: u8 = 1;
const HL_REGID: u8 = 2;
const SP_REGID: u8 = 3;

const DD_NAMES: &'static [ &'static str ] = &["BC", "DE", "HL", "SP"];

// ALU operations
const ALU_ADD: u8 = 0;
const ALU_ADC: u8 = 1;
const ALU_SUB: u8 = 2;
const ALU_SBC: u8 = 3;
const ALU_AND: u8 = 4;
const ALU_XOR: u8 = 5;
const ALU_OR : u8 = 6;
const ALU_CP : u8 = 7;

const ALU_NAMES: &'static [ &'static str ] = &["ADD", "ADC", "SUB", "SBC", "AND", "XOR", "OR", "CP"];

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

impl Regs {

}

pub struct Cpu {
    regs: Regs,
    mem: Mem,
    cycle: usize,
    halted: bool,
}

impl Cpu {
    pub fn new(cart: Cart) -> Cpu {
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
            mem: Mem::new(cart),
            cycle: 0,
            halted: false,
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

    fn set_reg8_by_id(&mut self, id: u8, value: u8) {
        match id {
            B_REGID => self.regs.b = value,
            C_REGID => self.regs.c = value,
            D_REGID => self.regs.d = value,
            E_REGID => self.regs.e = value,
            H_REGID => self.regs.h = value,
            L_REGID => self.regs.l = value,
            IND_HL_REGID => self.mem.write((self.regs.h as u16)<<8 | self.regs.l as u16, value),
            A_REGID => self.regs.a = value,
            _ => panic!("Wrong reg id")
        }
    }

    fn get_reg8_by_id(&self, id: u8) -> u8 {
        match id {
            B_REGID => self.regs.b,
            C_REGID => self.regs.c,
            D_REGID => self.regs.d,
            E_REGID => self.regs.e,
            H_REGID => self.regs.h,
            L_REGID => self.regs.l,
            IND_HL_REGID => self.mem.read((self.regs.h as u16)<<8 | self.regs.l as u16),
            A_REGID => self.regs.a,
            _ => panic!("Wrong reg id")
        }
    }

    fn set_reg16_by_id(&mut self, id: u8, value: u16) {
        let valh = (value >> 8) as u8;
        let vall = value as u8;

        match id {
            BC_REGID => {self.regs.b = valh; self.regs.c = vall; },
            DE_REGID => {self.regs.d = valh; self.regs.e = vall; },
            HL_REGID => {self.regs.h = valh; self.regs.l = vall; },
            SP_REGID => self.regs.sp = value,
            _ => panic!("Wrong reg id")
        }
    }

    fn get_reg16_by_id(&self, id: u8) -> u16 {
        match id {
            BC_REGID => (self.regs.b as u16)<<8 | self.regs.c as u16,
            DE_REGID => (self.regs.d as u16)<<8 | self.regs.e as u16,
            HL_REGID => (self.regs.h as u16)<<8 | self.regs.l as u16,
            SP_REGID => self.regs.sp,
            _ => panic!("Wrong reg id")
        }
    }

    fn decode(&mut self) -> usize {
        let instr = self.mem.read(self.regs.pc);
        match instr {
            0x00 => self.nop(),
            0x20 => self.jr_nz_r8(),
            0xC3 => self.jp_nn(),
            0xCB => self.decode_cb(),
            0xFA => self.ld_a_ind_nn(),
            0x76 => self.halt(),
            0x10 => self.stop(),
            _ if instr&0xCF == 0x01 => self.ld_dd_nn((instr>>4)&0x03),
            _ if instr&0xCF == 0x02 => self.store8_ind(instr),
            _ if instr&0xC7 == 0x06 => self.ld_r_n((instr>>3)&0x7),
            _ if instr&0xC0 == 0x40 => self.ld_r_r((instr>>3)&0x7, instr&0x7),
            _ if instr&0xC0 == 0x80 => self.alu((instr>>3)&0x7, instr&0x7),
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

    fn halt(&mut self) -> usize {
        println!("{:04x}: HALT", self.regs.pc);
        self.regs.pc += 1;
        4
    }

    fn stop(&mut self) -> usize {
        println!("{:04x}: STOP", self.regs.pc);
        self.regs.pc += 1;
        4
    }

    fn jp_nn(&mut self) -> usize{
        let address = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
        println!("{:04x}: JP ${:04x}", self.regs.pc, address);

        self.regs.pc = address;

        12
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

    fn ld_dd_nn(&mut self, reg_id: u8) -> usize {
        let value = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
        println!("{:04x}: LD {}, ${:04x}", self.regs.pc, DD_NAMES[reg_id as usize], value);
        self.regs.pc += 3;


        self.set_reg16_by_id(reg_id, value);

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

    fn alu(&mut self, operation: u8, reg_id: u8) -> usize {
        println!("{:04x}: {} {}", self.regs.pc, ALU_NAMES[operation as usize], REG_NAMES[reg_id as usize]);
        self.regs.pc += 1;

        let reg = self.get_reg8_by_id(reg_id) as u16;
        let a = self.regs.a as u16;
        let result;
        match operation {
            ALU_ADD => {
                result = a + reg;
                self.set_flag(HALF_CARRY_FLAG, result&0x10 != 0);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result == 0);
                self.set_flag(CARRY_FLAG, result&0x800 != 0);
            },
            ALU_ADC => {
                result = a + reg + (if self.get_flag(CARRY_FLAG) { 1 } else { 0 });
                self.set_flag(HALF_CARRY_FLAG, result&0x10 != 0);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result == 0);
                self.set_flag(CARRY_FLAG, result&0x800 != 0);
            },
            ALU_SUB => {
                result = a + !reg + 1;
                self.set_flag(HALF_CARRY_FLAG, result&0x10 != 0);
                self.set_flag(SUBSTRACT_FLAG, true);
                self.set_flag(ZERO_FLAG, result == 0);
                self.set_flag(CARRY_FLAG, result&0x800 != 0);
            },
            ALU_SBC => {
                result = a + !reg + 1 + (if self.get_flag(CARRY_FLAG) { 0xff } else { 0 });
                self.set_flag(HALF_CARRY_FLAG, result&0x10 != 0);
                self.set_flag(SUBSTRACT_FLAG, true);
                self.set_flag(ZERO_FLAG, result == 0);
                self.set_flag(CARRY_FLAG, result&0x800 != 0);
            },
            ALU_AND => {
                result = a & reg;
                self.set_flag(HALF_CARRY_FLAG, true);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result == 0);
                self.set_flag(CARRY_FLAG, false);
            },
            ALU_XOR => {
                result = a ^ reg;
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result == 0);
                self.set_flag(CARRY_FLAG, false);
            },
            ALU_OR => {
                result = a | reg;
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result == 0);
                self.set_flag(CARRY_FLAG, false);
            },
            ALU_CP => {
                result = a;
                self.set_flag(HALF_CARRY_FLAG, true);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, a == result);
                self.set_flag(CARRY_FLAG, false);
            },
            _ => panic!("wrong ALU operation"),
        }

        self.regs.a = result as u8;

        if reg_id == IND_HL_REGID { 8 } else { 4 }
    }

    fn ld_r_r(&mut self, dest_reg:u8, src_reg:u8) -> usize {
        println!("{:04x}: LD {}, {}", self.regs.pc, REG_NAMES[dest_reg as usize], REG_NAMES[src_reg as usize]);
        self.regs.pc += 1;

        let value = self.get_reg8_by_id(src_reg);
        self.set_reg8_by_id(dest_reg, value);

        if dest_reg == IND_HL_REGID || src_reg == IND_HL_REGID { 8 } else { 4 }
    }

    fn ld_r_n(&mut self, dest_reg: u8) -> usize {
        let value = self.mem.read(self.regs.pc+1);
        println!("{:04x}: LD {}, ${:02x}", self.regs.pc, REG_NAMES[dest_reg as usize], value);
        self.regs.pc += 2;

        self.set_reg8_by_id(dest_reg, value);

        if dest_reg == IND_HL_REGID { 12 } else { 8 }
    }

    fn ld_a_ind_nn(&mut self) -> usize {
        let address = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
        println!("{:04x}: LD A, (${:04x})", self.regs.pc, address);
        self.regs.pc += 3;

        self.regs.a = self.mem.read(address);

        16
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
