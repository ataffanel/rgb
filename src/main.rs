
#![allow(dead_code)]
#![allow(unused_variables)]

use std::env;

// use cart::Cart;

extern crate sdl2;
use sdl2::Sdl;
use sdl2::event::Event;
//use sdl2::event::Event::*;
use sdl2::keyboard::Keycode;

mod cart;
mod cpu;
//mod instructions;
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
    emulator_loop(cpu, disp, sdl);

    println!("Exiting ...");
}

fn emulator_loop(mut cpu: cpu::Cpu, mut disp: display::Display, mut sdl: Sdl) {
    'outer: loop {
        cpu.step();
        cpu.mem.video.step(cpu.cycle);

        if cpu.mem.video.image_ready {
            // Display the picture!
            disp.render_screen(&cpu.mem.video.screen);
        }
        while let Some(ev) = sdl.event_pump().poll_event() {
            match ev {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'outer,
                Event::Quit { .. } => break 'outer,
                _ => {}
            }
        }

        //if cpu.cycle >= 40000000 {break;}//40000000 {break;}
    }

    println!("PC: {:04X}", cpu.get_pc());
    cpu.print_regs();
    println!("Interrupts: {:04X}", cpu.mem.interrupts);
    dump_ram("vram.bin", &cpu.mem.video.vram);
    dump_ram("workram.bin", &cpu.mem.work);
    dump_memory_space("memory_space.bin", &cpu.mem);
}

use std::io::prelude::*;
use std::fs::File;

fn dump_ram(filename: &str, vram: &[u8]) {
    let mut f = File::create(filename).unwrap();
    f.write_all(vram).unwrap();
}

fn dump_memory_space(filename: &str, mem: &mem::Mem) {
    let mut f = File::create(filename).unwrap();
    for addr in 0 .. 65536 {
        f.write(&[mem.read(addr as u16)]).unwrap();
    }
}
