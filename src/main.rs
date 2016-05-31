
#![allow(dead_code)]
#![allow(unused_variables)]

use std::env;

// use cart::Cart;

mod cart;
mod cpu;
mod mem;

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

    let mut cpu = cpu::Cpu::new(cart);

    println!("Starting execution at address 0x1000: ");
    cpu.set_pc(0); //x100);
    cpu.execute_until(10000000);
}
