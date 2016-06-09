
#![allow(dead_code)]
#![allow(unused_variables)]

use std::env;

// use cart::Cart;

extern crate sdl2;

mod cart;
mod cpu;
mod mem;
mod video;
mod display;

fn main() {
    if env::args().len() != 2 {
        println!("Usage: gbc <path_to_rom>");
        return;
    }

    let rom_path = env::args().last().unwrap();

    println!("Loading rom {:?}", rom_path);

    let cart = cart::Cart::load(&rom_path);
    match cart {
        Ok(_) => (),
        Err(err) => {
            println!("Error reading rom: {}", err.error);
            return;
        },
    }

    let cart = cart.unwrap();

    // println!("Rom size: {}", cart.rom.len());
    // println!("First byte {:x}", cart.read(0));
    // println!("First byte {:x}", cart.read(0x8000));

    let (disp,sdl) = display::Display::new();

    let mut cpu = cpu::Cpu::new(cart);

    println!("Starting execution of bootstrap: ");
    cpu.reset();
    emulator_loop(cpu, disp);

    println!("Exiting ...");
}

static BLACK : [u8; 144*160*3] = [0; 144*160*3];

fn emulator_loop(mut cpu: cpu::Cpu, mut disp: display::Display) {
    loop {
        cpu.step();
        if cpu.cycle >= 1000000 {break;}
    }
}
