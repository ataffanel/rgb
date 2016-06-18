
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
mod bootstrap;

fn main() {
    if env::args().len() != 3 {
        println!("Usage: gbc <path to bootstrap> <path_to_rom>");
        return;
    }

    let bootstrap_path = env::args().nth(1).unwrap();
    println!("Loading bootstrap {:?}", bootstrap_path);
    let bootstrap = match bootstrap::Bootstrap::load(&bootstrap_path) {
        Ok(b) => b,
        Err(err) => {
            println!("Error reading bootstrap: {}", err.error);
            return;
        }
    };

    let rom_path = env::args().nth(2).unwrap();

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

    let mut cpu = cpu::Cpu::new(bootstrap, cart);

    println!("Starting execution of bootstrap: ");
    cpu.reset();
    //cpu.set_pc(0x100);
    emulator_loop(cpu, disp);

    println!("Exiting ...");
}

fn emulator_loop(mut cpu: cpu::Cpu, mut disp: display::Display) {
    loop {
        cpu.step();
        cpu.mem.video.step(cpu.cycle);

        if cpu.mem.video.image_ready {
            // Display the picture!
            disp.render_screen(&cpu.mem.video.screen);
        }

        if cpu.cycle >= 40000000 {break;}
    }

    println!("PC: {:04X}", cpu.get_pc());
    cpu.print_regs();
    dump_vram(&cpu.mem.video.vram);
}

use std::io::prelude::*;
use std::fs::File;

fn dump_vram(vram: &[u8]) {
    let mut f = File::create("vram.bin").unwrap();
    f.write_all(vram).unwrap();
}
