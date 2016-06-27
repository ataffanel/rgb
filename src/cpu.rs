// Emulation of the Game Boy LR35902 cpu

use bootstrap::Bootstrap;
use cart::Cart;
use mem::Mem;

use std::fmt;

#[cfg(feature="trace_cpu")]
macro_rules! trace {
    ( $($x:expr), * ) => {
        println!(
            $(
                $x,
            )*
        )
    }
}

#[cfg(not(feature="trace_cpu"))]
macro_rules! trace {
    ( $($x:expr), * ) => ()
}

pub const IRQ_VBLANK: u8 = 0x01;
pub const IRQ_LCDSTAT: u8 = 0x02;
pub const IRQ_TIMER: u8 = 0x04;
pub const IRQ_SERIAL: u8 = 0x08;
pub const IRQ_JOYPAD: u8 = 0x10;

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

// Jump conditions
const COND_NZ: u8 = 0;
const COND_Z : u8 = 1;
const COND_NC: u8 = 2;
const COND_C : u8 = 3;

const COND_NAMES: &'static [ &'static str ] = &["NZ", "Z", "NC", "C"];

// BC ALU operations
const BCALU_RLC :u8 = 0;
const BCALU_RRC :u8 = 1;
const BCALU_RL  :u8 = 2;
const BCALU_RR  :u8 = 3;
const BCALU_SLA :u8 = 4;
const BCALU_SRA :u8 = 5;
const BCALU_SWAP:u8 = 6;
const BCALU_SRL :u8 = 7;

const BCALU_NAMES: &'static [ &'static str ] = &["RLC", "RRC", "RL", "RR", "SLA", "SRA", "SWAP", "SRL"];

#[derive(PartialEq)]
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
        debug.push_str(&format!("| SP: {:04x}\n", self.sp));
        debug.push_str(&format!("| PC: {:04x}\n", self.pc));
        write!(f, "{}", debug)
    }
}

impl Regs {

}

pub struct Cpu {
    regs: Regs,
    pub mem: Mem,
    pub cycle: usize,
    halted: bool,
    stoped: bool,
    interrupts_enabled: bool,
    interrupts_enabled_next: bool,
}

impl Cpu {
    pub fn new(bootstrap: Bootstrap, cart: Cart) -> Cpu {
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
            mem: Mem::new(bootstrap, cart),
            cycle: 0,
            halted: false,
            stoped: false,
            interrupts_enabled_next: false,
            interrupts_enabled: false,
        }
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.regs.pc = pc;
    }

    pub fn get_pc(&self) -> u16 { self.regs.pc }

    pub fn execute_until(&mut self, cycle: usize) {
        while self.cycle < cycle {
            self.cycle += self.decode();
            trace!("{:?}", self.regs);
        }
    }

    pub fn step(&mut self) {
        if (self.mem.reg_ie & self.mem.reg_if) != 0 {
            self.stoped = false;
            self.halted = false;
        }

        if self.interrupts_enabled && (self.mem.reg_ie & self.mem.reg_if) != 0 {
            let int = self.mem.reg_ie&self.mem.reg_if;

            if int&IRQ_VBLANK != 0 {
                self.interrupt(0x40);
                self.mem.reg_if &= !IRQ_VBLANK;
            }
            if int&IRQ_LCDSTAT != 0 {
                self.interrupt(0x48);
                self.mem.reg_if &= !IRQ_LCDSTAT;
            }
            if int&IRQ_TIMER != 0 {
                self.interrupt(0x50);
                self.mem.reg_if &= !IRQ_TIMER;
            }
            if int&IRQ_SERIAL != 0 {
                self.interrupt(0x58);
                self.mem.reg_if &= !IRQ_SERIAL;
            }
            if int&IRQ_JOYPAD != 0 {
                self.interrupt(0x60);
                self.mem.reg_if &= !IRQ_JOYPAD;
            }

            self.cycle += 5*4;
        } else if !self.stoped && !self.halted {
            self.cycle += self.decode();
        } else {
            trace!("{}", if self.stoped {"Stopped!"} else {"Halted!"});
            self.cycle += 4;
        }

        trace!("{:?}", self.regs);
    }

    pub fn reset(&mut self) {
        self.regs.pc = 0;
    }

    pub fn get_cycle(&self) -> usize { self.cycle }

    pub fn print_regs(&self) { println!("{:?}", self.regs); }

    // Pivate methods

    fn interrupt(&mut self, address: u8) {
        trace!("{:04x}: INTERRUPT ${:02x}", self.regs.pc, address);

        self.mem.write(self.regs.sp-1, (self.regs.pc>>8) as u8);
        self.mem.write(self.regs.sp-2, self.regs.pc as u8);
        self.regs.sp -= 2;

        self.regs.pc = address as u16;
    }

    fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.regs.f |= flag;
        } else {
            self.regs.f &= !flag;
        }
    }

    fn get_flag(&self, flag: u8) -> bool { (self.regs.f & flag) != 0 }

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

    fn test_condition(&self, condition: u8) -> bool {
        match condition {
            COND_NZ => !self.get_flag(ZERO_FLAG),
            COND_Z  => self.get_flag(ZERO_FLAG),
            COND_NC => !self.get_flag(CARRY_FLAG),
            COND_C  => self.get_flag(CARRY_FLAG),
            _ => panic!("Wrong condition code"),
        }
    }

    fn decode(&mut self) -> usize {
        let interrupts_enabled_next = self.interrupts_enabled_next;

        let instr = self.mem.read(self.regs.pc);
        let cycle = match instr {
            0x00 => self.nop(),
            0xC3 => self.jp(true, false, 0),
            0xE9 => self.jp(false, false, 0),
            0xCB => self.decode_cb(),
            0xFA => self.ld_a_ind_nn(),
            0x08 => self.ld_ind_a16_sp(),
            0x76 => self.halt(),
            0x10 => self.stop(),
            0x18 => self.jr(false, 0),
            0xCD => self.call(false, 0),
            0xC9 => self.ret(false, 0),
            0xD9 => self.reti(),
            0x27 => self.daa(),
            0x2F => self.cpl(),
            0x37 => self.scf(),
            0x3f => self.ccf(),
            0xE8 => self.add_sp_r8(),
            0xF8 => self.ld_hl_sp_r8(),
            0xF9 => self.ld_sp_hl(),
            0xFE => self.cp_d8(),
            _ if instr&0xE7 == 0x20 => self.jr(true, (instr>>3)&0x03),
            _ if instr&0xE7 == 0xC2 => self.jp(true, true, (instr>>3)&0x03),
            _ if instr&0xCF == 0x01 => self.ld_dd_nn((instr>>4)&0x03),
            _ if instr&0xC7 == 0x02 => self.ld_ind(false, instr&0x08==0, (instr>>4)&0x03),
            _ if instr&0xEF == 0xEA => self.ld_ind(true, instr&0x10==0, 0),
            _ if instr&0xC7 == 0x06 => self.ld_r_n((instr>>3)&0x7),
            _ if instr&0xC0 == 0x40 => self.ld_r_r((instr>>3)&0x7, instr&0x7),
            _ if instr&0xC0 == 0x80 => self.alu(false, (instr>>3)&0x7, instr&0x7),
            _ if instr&0xC7 == 0xC6 => self.alu(true, (instr>>3)&0x7, 0),
            _ if instr&0xC7 == 0xC7 => self.rst(instr&0x38),
            _ if instr&0xC7 == 0x03 => self.inc_dec_dd(instr&0x08==0, (instr>>4)&0x03),
            _ if instr&0xC6 == 0x04 => self.inc_dec_r(instr&0x01==0, (instr>>3)&0x07),
            _ if instr&0xED == 0xE0 => self.ldh(instr&0x02==0, instr&0x10==0),
            _ if instr&0xC7 == 0xC4 => self.call(true, (instr>>3)&0x03),
            _ if instr&0xE7 == 0xC0 => self.ret(true, (instr>>3)&0x03),
            _ if instr&0xCF == 0xC1 => self.push_pop_qq(true, (instr>>4)&0x03),
            _ if instr&0xCF == 0xC5 => self.push_pop_qq(false, (instr>>4)&0x03),
            _ if instr&0xE7 == 0x07 => self.rotate((instr>>3)&0x03),
            _ if instr&0xF7 == 0xF3 => self.dei(instr&0x08 != 0),
            _ if instr&0xCF == 0x09 => self.add_hl_ss((instr&0x30)>>4),
            _ => {
                trace!("\n{:?}", self.regs);
                panic!("Invalid instruction op: 0x{:02x} at addr 0x{:04x}!", instr, self.regs.pc)
            }
        };
        self.interrupts_enabled = interrupts_enabled_next;

        cycle
    }

    fn nop(&mut self) -> usize {
        trace!("{:04x}: NOP", self.regs.pc);
        self.regs.pc += 1;
        4
    }

    #[allow(unreachable_code)]
    fn halt(&mut self) -> usize {
        trace!("{:04x}: HALT", self.regs.pc);
        self.halted = true;
        self.regs.pc += 1;
        //panic!("Halt not implemented!");
        4
    }

    #[allow(unreachable_code)]
    fn stop(&mut self) -> usize {
        trace!("{:04x}: STOP", self.regs.pc);
        self.stoped = true;
        self.regs.pc += 1;
        //panic!("STOP not implemented!");
        4
    }

    fn dei(&mut self, enable: bool) -> usize {
        trace!("{:04x}: {}", self.regs.pc, if enable {"EI"} else {"DI"});
        self.interrupts_enabled_next = enable;
        self.regs.pc += 1;
        4
    }

    fn jp(&mut self, immediate: bool, conditional: bool, condition: u8) -> usize {
        let address;
        if immediate{
            address = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
            let cond_str = if conditional { format!("{}, ", COND_NAMES[condition as usize]) } else { String::from("") };
            trace!("{:04x}: JP {}${:04x}", self.regs.pc, cond_str, address);
        } else {
            address = self.get_reg16_by_id(HL_REGID);
            trace!("{:04x}: JP (HL)", self.regs.pc);
        }

        if !conditional || self.test_condition(condition) {
            self.regs.pc = address;

            if immediate {16} else {4}
        } else {
            self.regs.pc += 3;
            12
        }
    }

    fn jr(&mut self, conditional: bool, condition: u8) -> usize {
        let val = self.mem.read(self.regs.pc+1) as i8;
        let cond_str = if conditional { format!("{}, ", COND_NAMES[condition as usize]) } else { String::from("") };
        trace!("{:04x}: JR {}{}", self.regs.pc, cond_str, val);
        self.regs.pc += 2;
        if !conditional || self.test_condition(condition) {
            let newpc = (self.regs.pc as i32) + (val as i32);
            self.regs.pc = newpc as u16;
            12
        } else {
            8
        }
    }

    fn rst(&mut self, address: u8) -> usize {
        trace!("{:04x}: RST ${:02x}", self.regs.pc, address);
        self.regs.pc += 1;
        self.mem.write(self.regs.sp-1, (self.regs.pc>>8) as u8);
        self.mem.write(self.regs.sp-2, self.regs.pc as u8);
        self.regs.sp -= 2;
        self.regs.pc = address as u16;

        16
    }

    fn call(&mut self, conditional: bool, condition: u8) -> usize {
        let newpc = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
        let cond_str = if conditional { format!("{}, ", COND_NAMES[condition as usize]) } else { String::from("") };
        trace!("{:04x}: CALL {}${:04x}", self.regs.pc, cond_str, newpc);
        self.regs.pc += 3;
        if !conditional || self.test_condition(condition) {
            self.mem.write(self.regs.sp-1, (self.regs.pc>>8) as u8);
            self.mem.write(self.regs.sp-2, self.regs.pc as u8);
            self.regs.sp -= 2;
            self.regs.pc = newpc;
            24
        } else {
            12
        }
    }

    fn ret(&mut self, conditional: bool, condition: u8) -> usize {
        let cond_str = if conditional { format!(" {}", COND_NAMES[condition as usize]) } else { String::from("") };
        trace!("{:04x}: RET{}", self.regs.pc, cond_str);
        self.regs.pc += 1;
        if !conditional || self.test_condition(condition) {
            let pc_l = self.mem.read(self.regs.sp);
            let pc_h = self.mem.read(self.regs.sp+1);
            self.regs.sp += 2;
            self.regs.pc = (pc_h as u16)<<8 | pc_l as u16;
            if conditional { 20 } else { 16 }
        } else {
            8
        }
    }

    fn reti(&mut self) -> usize {
        trace!("{:04x}: RETI", self.regs.pc);

        let pc_l = self.mem.read(self.regs.sp);
        let pc_h = self.mem.read(self.regs.sp+1);
        self.regs.sp += 2;
        self.regs.pc = (pc_h as u16)<<8 | pc_l as u16;

        self.interrupts_enabled = true;
        self.interrupts_enabled_next = true;

        16
    }

    #[allow(unused_assignments)]
    fn ldh(&mut self, immediate: bool, store: bool) -> usize {
        let address;
        let addr_str;
        if immediate {
            address = self.mem.read(self.regs.pc+1);
            addr_str = format!("${:02x}", address);
            self.regs.pc += 2;
        } else {
            address = self.regs.c;
            addr_str = String::from("C");
            self.regs.pc += 1;
        }

        if store {
            trace!("{:04x}: LD ($FF00+{}), A", self.regs.pc, addr_str);
            self.mem.write(0xff00 + (address as u16), self.regs.a);
        } else {
            trace!("{:04x}: LD A, ($FF00+{})", self.regs.pc, addr_str);
            self.regs.a = self.mem.read(0xff00 + (address as u16));
        }

        if immediate { 12 } else { 8 }
    }

    fn ld_ind_a16_sp(&mut self) -> usize {
        let addr = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
        trace!("{:04x}: LD (${:04x}), SP", self.regs.pc, addr);
        self.regs.pc += 3;

        self.mem.write(addr, (self.regs.sp&0xff) as u8);
        self.mem.write(addr.wrapping_add(1), (self.regs.sp>>8) as u8);

        20
    }

    fn ld_dd_nn(&mut self, reg_id: u8) -> usize {
        let value = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
        trace!("{:04x}: LD {}, ${:04x}", self.regs.pc, DD_NAMES[reg_id as usize], value);
        self.regs.pc += 3;

        self.set_reg16_by_id(reg_id, value);

        12
    }

    fn ld_sp_hl(&mut self) -> usize {
        trace!("{:04x}: LD SP, HL", self.regs.pc);
        self.regs.pc += 1;

        self.regs.sp = self.get_reg16_by_id(HL_REGID);

        8
    }

    fn ld_ind(&mut self, immediate: bool, store: bool, reg_id: u8) -> usize {
        let address: u16;

        let (address, reg_name) = if immediate {
            let addr = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
            (addr, format!("${:04x}", addr))
        } else {
            match reg_id {
                0 => ((self.regs.b as u16) << 8 | self.regs.c as u16, "BC".to_string()),
                1 => ((self.regs.d as u16) << 8 | self.regs.e as u16, "DE".to_string()),
                2 => {
                    let hl = (self.regs.h as u16) << 8 | self.regs.l as u16;
                    let new_hl = hl+1;
                    self.regs.h = (new_hl >> 8) as u8;
                    self.regs.l = new_hl as u8;

                    (hl, "HL+".to_string())
                },
                3 => {
                    let hl = (self.regs.h as u16) << 8 | self.regs.l as u16;
                    let new_hl = hl.wrapping_sub(1);
                    self.regs.h = (new_hl >> 8) as u8;
                    self.regs.l = new_hl as u8;

                    (hl, "HL-".to_string())
                },
                _ => panic!("Bug in decoding")
            }
        };

        if store {
            self.mem.write(address, self.regs.a);
            trace!("{:04x}: LD ({}), A", self.regs.pc, reg_name);
        } else {
            self.regs.a = self.mem.read(address);
            trace!("{:04x}: LD A, ({})", self.regs.pc, reg_name);
        }


        if immediate {
            self.regs.pc += 3;
            16
        } else {
            self.regs.pc += 1;
            8
        }
    }

    fn push_pop_qq(&mut self, pop: bool, reg_id: u8) -> usize {
        let (reg_h, reg_l, reg_name) = match reg_id {
            0 => (&mut self.regs.b, &mut self.regs.c, "BC"),
            1 => (&mut self.regs.d, &mut self.regs.e, "DE"),
            2 => (&mut self.regs.h, &mut self.regs.l, "HL"),
            3 => (&mut self.regs.a, &mut self.regs.f, "AF"),
            _ => panic!("Bug in decoding")
        };

        if pop {
            if reg_id == 3 {
                *reg_l = self.mem.read(self.regs.sp)&0xf0;
            } else {
                *reg_l = self.mem.read(self.regs.sp);
            }
            *reg_h = self.mem.read(self.regs.sp.wrapping_add(1));
            self.regs.sp = self.regs.sp.wrapping_add(2);
            trace!("{:04x}: POP {}", self.regs.pc, reg_name);
        } else {
            self.mem.write(self.regs.sp.wrapping_sub(1), *reg_h);
            self.mem.write(self.regs.sp.wrapping_sub(2), *reg_l);
            self.regs.sp = self.regs.sp.wrapping_sub(2);
            trace!("{:04x}: PUSH {}", self.regs.pc, reg_name);
        }


        self.regs.pc += 1;
        if pop { 12 } else { 16 }
    }

    fn daa(&mut self) -> usize {
        trace!("{:04x}: DAA", self.regs.pc);
        self.regs.pc += 1;

        // Get easy to use values
        let low = self.regs.a & 0x0f;
        let high = self.regs.a >> 4;
        let h = self.get_flag(HALF_CARRY_FLAG);
        let c = self.get_flag(CARRY_FLAG);
        let n = self.get_flag(SUBSTRACT_FLAG);

        let diff: u8 = if low < 10 && high < 10 && !h && !c {
            0
        } else if (low < 10 && high < 10 && h && !c) ||
                  (low > 9 && high < 9 && !c) {
            0x06
        } else if low < 10 && !h && (high > 9 || c) {
            0x60
        } else {
            0x66
        };

        if n {
            //diff = (!diff).wrapping_add(1);
            self.regs.a = self.regs.a.wrapping_sub(diff);
        } else {
            self.regs.a = self.regs.a.wrapping_add(diff);
        }



        self.set_flag(CARRY_FLAG, c || (low > 9 && high > 8) || (low < 9 && high > 9));
        self.set_flag(HALF_CARRY_FLAG, low > 9 || (n && h && low < 6));

        4
    }

    fn rotate(&mut self, operation: u8) -> usize {
        trace!("{:04x}: {}A", self.regs.pc, BCALU_NAMES[operation as usize]);
        self.regs.pc += 1;

        let mut value = self.regs.a;
        match operation {
            BCALU_RLC => {
                self.set_flag(CARRY_FLAG, value&0x80 != 0);
                value <<= 1;
                if self.get_flag(CARRY_FLAG) {
                    value |= 0x01;
                }

            },
            BCALU_RRC => {
                    self.set_flag(CARRY_FLAG, value&0x01 != 0);
                    value >>= 1;
                    if self.get_flag(CARRY_FLAG) {
                        value |= 0x80;
                    }
                },
            BCALU_RL => {
                    let carry = self.get_flag(CARRY_FLAG);
                    self.set_flag(CARRY_FLAG, value&0x80 != 0);
                    value <<= 1;
                    if carry {
                        value |= 0x01;
                    }
                },
            BCALU_RR => {
                let carry = self.get_flag(CARRY_FLAG);
                self.set_flag(CARRY_FLAG, value&0x01 != 0);
                value >>= 1;
                if carry {
                    value |= 0x80;
                }
            },
            _ => panic!("Bug in decoding"),
        };
        self.regs.a = value;
        self.set_flag(HALF_CARRY_FLAG, false);
        self.set_flag(SUBSTRACT_FLAG, false);
        self.set_flag(ZERO_FLAG, false);

        4
    }

    fn add_sp_r8(&mut self) -> usize {
        let value = ((self.mem.read(self.regs.pc+1) as i8) as i16) as u16;
        trace!("{:04x}: ADD SP, {}", self.regs.pc, value);
        self.regs.pc += 2;

        let result = self.regs.sp.wrapping_add(value);
        let hresult = (self.regs.sp&0xff) + (value&0xff);
        let hhresult = (self.regs.sp&0x0f) + (value&0x0f);
        self.regs.sp = result as u16;

        self.set_flag(ZERO_FLAG, false);
        self.set_flag(SUBSTRACT_FLAG, false);
        self.set_flag(HALF_CARRY_FLAG, hhresult&0x10 != 0);
        self.set_flag(CARRY_FLAG, hresult&0x100 != 0);

        16
    }

    fn ld_hl_sp_r8(&mut self) -> usize {
        let ivalue = (self.mem.read(self.regs.pc+1) as i8) as i16;
        let value = ivalue as u16;
        trace!("{:04x}: LD HL, SP{:+}", self.regs.pc, ivalue);
        self.regs.pc += 2;

        let result = self.regs.sp.wrapping_add(value);
        let hresult = (self.regs.sp&0xff) + (value&0xff);
        let hhresult = (self.regs.sp&0x0f) + (value&0x0f);
        self.set_reg16_by_id(HL_REGID, result as u16);

        self.set_flag(ZERO_FLAG, false);
        self.set_flag(SUBSTRACT_FLAG, false);
        self.set_flag(HALF_CARRY_FLAG, hhresult&0x10 != 0);
        self.set_flag(CARRY_FLAG, hresult&0x100 != 0);

        12
    }

    fn add_hl_ss(&mut self, reg_id: u8) -> usize {
        trace!("{:04x}: ADD HL, {}", self.regs.pc, DD_NAMES[reg_id as usize]);
        self.regs.pc += 1;

        let hl = self.get_reg16_by_id(HL_REGID);
        let reg = self.get_reg16_by_id(reg_id);
        let value = reg as u32 + hl as u32;
        let hvalue = (hl&((1<<12)-1)) + (reg&((1<<12)-1));
        self.set_reg16_by_id(HL_REGID, value as u16);

        self.set_flag(SUBSTRACT_FLAG, false);
        self.set_flag(HALF_CARRY_FLAG, hvalue&(1<<12) != 0);
        self.set_flag(CARRY_FLAG, value&0x10000 != 0);

        8
    }

    fn cp_d8(&mut self) -> usize {
        let value = self.mem.read(self.regs.pc+1);
        trace!("{:04x}: CP ${}", self.regs.pc, value);
        self.regs.pc += 2;

        let a = self.regs.a;
        self.set_flag(CARRY_FLAG, a < value);
        self.set_flag(ZERO_FLAG, a == value);
        self.set_flag(HALF_CARRY_FLAG, (a&0x0f) < (value&0x0f));
        self.set_flag(SUBSTRACT_FLAG, true);

        8
    }

    fn cpl(&mut self) -> usize {
        trace!("{:04x}: CPL", self.regs.pc);
        self.regs.pc += 1;

        self.regs.a = !self.regs.a;
        self.set_flag(SUBSTRACT_FLAG, true);
        self.set_flag(HALF_CARRY_FLAG, true);

        4
    }

    fn ccf(&mut self) -> usize {
        trace!("{:04x}: CPL", self.regs.pc);
        self.regs.pc += 1;

        let cy = self.get_flag(CARRY_FLAG);
        self.set_flag(CARRY_FLAG, !cy);
        self.set_flag(SUBSTRACT_FLAG, false);
        self.set_flag(HALF_CARRY_FLAG, false);

        4
    }

    fn scf(&mut self) -> usize {
        trace!("{:04x}: SCF", self.regs.pc);
        self.regs.pc += 1;

        self.set_flag(CARRY_FLAG, true);
        self.set_flag(SUBSTRACT_FLAG, false);
        self.set_flag(HALF_CARRY_FLAG, false);

        4
    }

    fn alu(&mut self, immediate: bool, operation: u8, reg_id: u8) -> usize {
        let val: u16;
        if immediate {
            val = self.mem.read(self.regs.pc+1) as u16;
            trace!("{:04x}: {} ${:02x}", self.regs.pc, ALU_NAMES[operation as usize], val);
            self.regs.pc += 2;
        } else {
            val = self.get_reg8_by_id(reg_id) as u16;
            trace!("{:04x}: {} {}", self.regs.pc, ALU_NAMES[operation as usize], REG_NAMES[reg_id as usize]);
            self.regs.pc += 1;
        }
        let a = self.regs.a as u16;
        let result: u16;
        match operation {
            ALU_ADD => {
                result = a + val;
                let hresult = (a&0x0f) + (val&0x0f);
                self.set_flag(HALF_CARRY_FLAG, hresult&0x10 != 0);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result&0xff == 0);
                self.set_flag(CARRY_FLAG, result&0x100 != 0);
            },
            ALU_ADC => {
                result = a + val + (if self.get_flag(CARRY_FLAG) { 1 } else { 0 });
                let hresult = (a&0x0f) + (val&0x0f)+ (if self.get_flag(CARRY_FLAG) { 1 } else { 0 });
                self.set_flag(HALF_CARRY_FLAG, hresult&0x10 != 0);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result&0xff == 0);
                self.set_flag(CARRY_FLAG, result&0x100 != 0);
            },
            ALU_SUB => {
                result = a + !(val as u8) as u16 + 1;
                self.set_flag(HALF_CARRY_FLAG, (a&0x0f) < (val&0x0f));
                self.set_flag(SUBSTRACT_FLAG, true);
                self.set_flag(ZERO_FLAG, result&0xff == 0);
                self.set_flag(CARRY_FLAG, a<val);
            },
            ALU_SBC => {
                let c = self.get_flag(CARRY_FLAG);
                result = a + !(val as u8) as u16 + 1 + (if c { 0xff } else { 0 });
                self.set_flag(HALF_CARRY_FLAG, (a&0x0f) < (val&0x0f) + (if c { 1 } else { 0 }));
                self.set_flag(SUBSTRACT_FLAG, true);
                self.set_flag(ZERO_FLAG, result&0xff == 0);
                self.set_flag(CARRY_FLAG, a < val + (if c { 1 } else { 0 }));
            },
            ALU_AND => {
                result = a & val;
                self.set_flag(HALF_CARRY_FLAG, true);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result&0xff == 0);
                self.set_flag(CARRY_FLAG, false);
            },
            ALU_XOR => {
                result = a ^ val;
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result&0xff == 0);
                self.set_flag(CARRY_FLAG, false);
            },
            ALU_OR => {
                result = a | val;
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(ZERO_FLAG, result&0xff == 0);
                self.set_flag(CARRY_FLAG, false);
            },
            ALU_CP => {
                result = a;
                self.set_flag(HALF_CARRY_FLAG, (a&0x0f) < (val&0x0f));
                self.set_flag(SUBSTRACT_FLAG, true);
                self.set_flag(ZERO_FLAG, a == val);
                self.set_flag(CARRY_FLAG, a<val);
            },
            _ => panic!("wrong ALU operation"),
        }

        self.regs.a = result as u8;

        if reg_id == IND_HL_REGID || immediate { 8 } else { 4 }
    }

    fn inc_dec_dd(&mut self, inc:bool, reg_id: u8) -> usize {
        trace!("{:04x}: {} {}", self.regs.pc, if inc {"INC"} else {"DEC"}, DD_NAMES[reg_id as usize]);
        self.regs.pc += 1;

        let result = self.get_reg16_by_id(reg_id);
        if inc {
            self.set_reg16_by_id(reg_id, result.wrapping_add(1));
        } else {
            self.set_reg16_by_id(reg_id, result.wrapping_sub(1));
        }

        8
    }

    fn inc_dec_r(&mut self, inc: bool, reg_id: u8) -> usize {
        trace!("{:04x}: {} {}", self.regs.pc, if inc {"INC"} else {"DEC"}, REG_NAMES[reg_id as usize]);
        self.regs.pc += 1;

        let reg = self.get_reg8_by_id(reg_id) as i16;
        let result;
        if inc {
            result = reg + 1;
            self.set_reg8_by_id(reg_id, result as u8);
            self.set_flag(SUBSTRACT_FLAG, false);
            let hresult = (reg&0x0f) + 1;
            self.set_flag(HALF_CARRY_FLAG, (hresult&0x10) != 0);
        } else {
            result = reg - 1;
            self.set_reg8_by_id(reg_id, result as u8);
            self.set_flag(SUBSTRACT_FLAG, true);
            self.set_flag(HALF_CARRY_FLAG, (reg&0x0f)<1);
        }
        self.set_flag(ZERO_FLAG, (result as u8) == 0);


        4
    }

    fn ld_r_r(&mut self, dest_reg:u8, src_reg:u8) -> usize {
        trace!("{:04x}: LD {}, {}", self.regs.pc, REG_NAMES[dest_reg as usize], REG_NAMES[src_reg as usize]);
        self.regs.pc += 1;

        let value = self.get_reg8_by_id(src_reg);
        self.set_reg8_by_id(dest_reg, value);

        if dest_reg == IND_HL_REGID || src_reg == IND_HL_REGID { 8 } else { 4 }
    }

    fn ld_r_n(&mut self, dest_reg: u8) -> usize {
        let value = self.mem.read(self.regs.pc+1);
        trace!("{:04x}: LD {}, ${:02x}", self.regs.pc, REG_NAMES[dest_reg as usize], value);
        self.regs.pc += 2;

        self.set_reg8_by_id(dest_reg, value);

        if dest_reg == IND_HL_REGID { 12 } else { 8 }
    }

    fn ld_a_ind_nn(&mut self) -> usize {
        let address = (self.mem.read(self.regs.pc+2) as u16)<<8 |  self.mem.read(self.regs.pc+1) as u16;
        trace!("{:04x}: LD A, (${:04x})", self.regs.pc, address);
        self.regs.pc += 3;

        self.regs.a = self.mem.read(address);

        16
    }

    fn decode_cb(&mut self) -> usize {
        self.regs.pc += 1;
        let instr = self.mem.read(self.regs.pc);

        4 + match instr {
            _ if instr&0xC0 == 0x00 => self.bc_alu((instr >> 3) & 0x07, instr & 0x07),
            _ if instr&0xC0 == 0x40 => self.bit((instr >> 3) & 0x07, instr & 0x07),
            _ if instr&0x80 == 0x80 => self.res_set(instr&0x40 == 0,(instr >> 3) & 0x07, instr & 0x07),
            _ => {
                trace!("\n{:?}", self.regs);
                panic!("Uknown prefixed instruction op: 0x{:02x} at addr 0x{:04x}!", instr, self.regs.pc)
            }
        }
    }

    fn bc_alu(&mut self, operation: u8, reg_id:u8) -> usize {
        trace!("{:04x}: {} {}", self.regs.pc-1, BCALU_NAMES[operation as usize], REG_NAMES[reg_id as usize]);
        self.regs.pc += 1;

        let mut value = self.get_reg8_by_id(reg_id);

        match operation {
            BCALU_RLC => {
                self.set_flag(CARRY_FLAG, value&0x80 != 0);
                value <<= 1;
                if self.get_flag(CARRY_FLAG) {
                    value |= 0x01;
                }
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
            },
    	    BCALU_RRC => {
                    self.set_flag(CARRY_FLAG, value&0x01 != 0);
                    value >>= 1;
                    if self.get_flag(CARRY_FLAG) {
                        value |= 0x80;
                    }
                    self.set_flag(HALF_CARRY_FLAG, false);
                    self.set_flag(SUBSTRACT_FLAG, false);
                },
    	    BCALU_RL => {
    		          let carry = self.get_flag(CARRY_FLAG);
                    self.set_flag(CARRY_FLAG, value&0x80 != 0);
                    value <<= 1;
                    if carry {
                        value |= 0x01;
                    }
                    self.set_flag(HALF_CARRY_FLAG, false);
                    self.set_flag(SUBSTRACT_FLAG, false);
                },
            BCALU_RR => {
                let carry = self.get_flag(CARRY_FLAG);
                self.set_flag(CARRY_FLAG, value&0x01 != 0);
                value >>= 1;
                if carry {
                    value |= 0x80;
                }
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
            },
            BCALU_SLA => {
                self.set_flag(CARRY_FLAG, value&0x80 != 0);
                value <<= 1;
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
            },
            BCALU_SRA => {
                let highset = value&0x80 != 0;
                self.set_flag(CARRY_FLAG, value&0x01 != 0);
                value >>= 1;
                if highset {
                    value |= 0x80;
                }
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
            },
            BCALU_SWAP => {
                value = (value << 4) | (value >> 4);
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
                self.set_flag(CARRY_FLAG, false);
            },
            BCALU_SRL => {
                self.set_flag(CARRY_FLAG, value&0x01 != 0);
                value >>= 1;
                self.set_flag(HALF_CARRY_FLAG, false);
                self.set_flag(SUBSTRACT_FLAG, false);
            },
            _ => panic!("Bug in decoding")
        };

        self.set_reg8_by_id(reg_id, value);
        self.set_flag(ZERO_FLAG, value&0xff == 0);

        if reg_id == IND_HL_REGID {12} else {4}
    }

    fn bit(&mut self, bit: u8, reg_id: u8) -> usize {
        trace!("{:04x}: BIT {}, {}", self.regs.pc-1, bit, REG_NAMES[reg_id as usize]);
        self.regs.pc += 1;

        let flag = self.get_reg8_by_id(reg_id) & (1<<bit) == 0;
        self.set_flag(ZERO_FLAG, flag);
        self.set_flag(HALF_CARRY_FLAG, true);
        self.set_flag(SUBSTRACT_FLAG, false);

        if reg_id == IND_HL_REGID {12} else {4}
    }

    fn res_set(&mut self, res: bool, bit: u8, reg_id: u8) -> usize {
        trace!("{:04x}: {} {}, {}", self.regs.pc-1, if res {"RES"} else {"SET"},
                                      bit, REG_NAMES[reg_id as usize]);
        self.regs.pc += 1;

        let mut value = self.get_reg8_by_id(reg_id);
        if res {
            value &= !(1<<bit);
        } else {
            value |= 1<<bit;
        }
        self.set_reg8_by_id(reg_id, value);

        if reg_id == IND_HL_REGID {12} else {4}
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    use super::Regs;
    //use mem::Mem;
    use bootstrap::Bootstrap;
    use cart::Cart;

    fn test_cpu(instructions: &[u8], nstep: usize, expected: Regs) -> Cpu {
        // Create an empty bootstrap and put the test code in the cart
        let bootstrap = Bootstrap::create_from_slice(&[]);
        let cart = Cart::create_from_slice(instructions);
        let mut cpu = Cpu::new(bootstrap, cart);

        // Turn bootstrap OFF
        cpu.mem.write(0xff50, 1);

        for _ in 0 .. nstep {
            cpu.step();
        }

        assert_eq!(cpu.regs, expected);

        cpu
    }

    #[test]
    fn nop() {
        test_cpu(&[0x00], 1, Regs{
            a: 0, b: 0,
            c: 0, d: 0,
            e: 0, f: 0,
            h: 0, l: 0,
            pc: 1,
            sp: 0,
        });
    }

    #[test]
    fn ld_ind_a() {
        test_cpu(&[0x3E, 0x42, 0x77, 0x80, 0xff, 0xA7], 3, Regs {
            a: 0, b: 0,
            c: 0, d: 0,
            e: 0, f: 0,
            h: 0, l: 0,
            pc: 1,
            sp: 0,
        });
    }
}
