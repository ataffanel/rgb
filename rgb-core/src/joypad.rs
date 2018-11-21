
// bit position in state variables
const STATE_RIGH: u8 = 1;
const STATE_LEFT: u8 = 2;
const STATE_UP: u8 = 4;
const STATE_DOWN: u8 = 8;

const STATE_A: u8 = 16;
const STATE_B: u8 = 32;
const STATE_SELECT: u8 = 64;
const STATE_START: u8 = 128;

pub enum JoypadButton {
    Down,
    Up,
    Left,
    Right,
    Start,
    Select,
    A,
    B,
}

pub struct Joypad {
    state: u8,

    reg: u8,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            state: 0xff,

            reg: 0x3f,
        }
    }

    // Memory access
    pub fn read(&self, _address: u16) -> u8 { self.reg }
    pub fn write(&mut self, _address: u16, data: u8) { self.reg = (self.reg&0xCF) | (data&0x30); }

    //emulator loop access
    pub fn set_button(&mut self, button: JoypadButton, pressed: bool) {
        let bit = match button {
            JoypadButton::Up => STATE_UP,
            JoypadButton::Down => STATE_DOWN,
            JoypadButton::Left => STATE_LEFT,
            JoypadButton::Right => STATE_RIGH,
            JoypadButton::Start => STATE_START,
            JoypadButton::Select => STATE_SELECT,
            JoypadButton::A => STATE_A,
            JoypadButton::B => STATE_B,
        };

        if pressed {
            self.state &= !bit;
        } else {
            self.state |= bit;
        }
    }

    pub fn step(&mut self) {
        let mut newreg = 0x0f;

        if self.reg&0x10 == 0 {
            newreg &= self.state&0x0f;
        }

        if self.reg&0x20 == 0 {
            newreg &= (self.state>>4)&0x0f;
        }

        newreg |= self.reg&0x30;

        // ToDo: check if there is a change and set the interrupt flag accordingly

        self.reg = newreg;
    }

}
