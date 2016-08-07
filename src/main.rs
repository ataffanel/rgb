
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
mod mem;
mod video;
mod display;
mod bootstrap;
mod joypad;
mod timer;

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

    println!("Loaded cartridge:\n{}", cart);

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
        cpu.mem.step();
        cpu.mem.reg_if |= cpu.mem.timer.step(cpu.cycle);
        cpu.mem.reg_if |= cpu.mem.video.step(cpu.cycle);
        cpu.mem.joypad.step();

        if cpu.mem.video.image_ready {
            // Display the picture!
            disp.render_screen(&cpu.mem.video.screen);

            while let Some(ev) = sdl.event_pump().poll_event() {
                match ev {
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'outer,
                    Event::KeyDown { keycode: Some(Keycode::Return), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Start, true),
                    Event::KeyDown { keycode: Some(Keycode::Up), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Up, true),
                    Event::KeyDown { keycode: Some(Keycode::Down), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Down, true),
                    Event::KeyDown { keycode: Some(Keycode::Left), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Left, true),
                    Event::KeyDown { keycode: Some(Keycode::Right), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Right, true),
                    Event::KeyDown { keycode: Some(Keycode::S), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::A, true),
                    Event::KeyDown { keycode: Some(Keycode::A), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::B, true),
                    Event::KeyUp { keycode: Some(Keycode::Return), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Start, false),
                    Event::KeyUp { keycode: Some(Keycode::Up), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Up, false),
                    Event::KeyUp { keycode: Some(Keycode::Down), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Down, false),
                    Event::KeyUp { keycode: Some(Keycode::Left), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Left, false),
                    Event::KeyUp { keycode: Some(Keycode::Right), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::Right, false),
                    Event::KeyUp { keycode: Some(Keycode::S), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::A, false),
                    Event::KeyUp { keycode: Some(Keycode::A), .. } => cpu.mem.joypad.set_button(joypad::JoypadButton::B, false),
                    Event::Quit { .. } => break 'outer,
                    _ => {}
                }
            }
        }
    }

    println!("PC: {:04X}", cpu.get_pc());
    cpu.print_regs();
    println!("IE: {:04X}", cpu.mem.reg_ie);
    println!("IF: {:04X}", cpu.mem.reg_if);
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
