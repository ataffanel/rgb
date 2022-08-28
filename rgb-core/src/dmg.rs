
use crate::cart::Cart;
use crate::cpu::Cpu;
use crate::bootstrap::Bootstrap;
use crate::joypad;

/// DMG emulator
///
/// This object contains and run all the elements of a DMG GameBoy: CPU, memory,
/// video and cart. It provides helper function to interact with the emulation.
///
/// The CPU and most of the objects in it are left public so that advanced
/// internal functionality can be accessed (for example to dump or modify
/// internal memories).
pub struct Dmg {
    pub cpu: Cpu,
}

impl Dmg {
    /// Create a DMG emulator attached to a cart
    pub fn new(cart: Cart) -> Self {
        Self::new_with_bootstrap(cart, Bootstrap::create_default())
    }

    /// Create a DMG emulator attached to a cart with a custom bootstrap
    pub fn new_with_bootstrap(cart: Cart, bootstrap: Bootstrap) -> Self {
        let cpu = Cpu::new(bootstrap, cart);

        Self { cpu }
    }

    /// Step the emulation one step
    ///
    /// This will run one CPU instruction, this means that it can result in 
    /// running more than one clock cycles.
    ///
    /// Return `true` if an new video frame is ready to display on that step.
    pub fn step(&mut self) -> bool {
        self.cpu.step();
        self.cpu.mem.step();
        self.cpu.mem.reg_if |= self.cpu.mem.timer.step(self.cpu.cycle);
        self.cpu.mem.reg_if |= self.cpu.mem.video.step(self.cpu.cycle);
        self.cpu.mem.joypad.step();
        self.cpu.mem.audio.step(self.cpu.cycle);

        self.cpu.mem.video.image_ready
    }

    /// Runs the emulation until a frame becomes available to display
    pub fn run_until_next_frame(&mut self) {
        while !self.step() {}
    }

    /// Returns a reference to the display framebuffer
    pub fn borrow_display(&self) -> &[u8] {
        &self.cpu.mem.video.screen
    }

    /// Reset the CPU
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    /// Set new state for an input button
    pub fn set_button(&mut self, button: joypad::JoypadButton, pressed: bool) {
        self.cpu.mem.joypad.set_button(button, pressed);
    }
}