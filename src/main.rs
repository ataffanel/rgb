
#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate clap;
use clap::App;

extern crate sdl2;
use sdl2::Sdl;
use sdl2::event::Event;
//use sdl2::event::Event::*;
use sdl2::keyboard::Keycode;
//use sdl2::joystick::JoystickSubsystem;
use sdl2::controller::Button;

mod cart;
mod cpu;
mod mem;
mod video;
mod display;
mod bootstrap;
mod joypad;
mod timer;

fn main() {
    let matches = App::new("rgb")
                          .version(crate_version!())
                          .author(crate_authors!())
                          .about("Gameboy emulator")
                          .args_from_usage(
                              "-b, --bootstrap=[bootstrap] 'Custom bootstrap rom'
                              <ROM>              'Gamboy rom to run'")
                          .get_matches();

    let bootstrap;
    if let Some(bootstrap_path) = matches.value_of("bootstrap") {
        println!("Loading bootstrap {:?}", bootstrap_path);
        bootstrap = match bootstrap::Bootstrap::load(&bootstrap_path.to_string()) {
            Ok(b) => b,
            Err(err) => {
                println!("Error reading bootstrap: {}", err.error);
                return;
            }
        };
    } else {
        panic!("Integrated bootstrap not implemented yet. Please set one.")
    }


    let rom_path = matches.value_of("ROM").unwrap();

    println!("Loading rom {:?}", rom_path);

    let cart = cart::Cart::load(&rom_path.to_string());
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

    let gamepad = sdl.game_controller().unwrap().open(0);

    let mut cpu = cpu::Cpu::new(bootstrap, cart);

    println!("Starting execution of bootstrap: ");
    cpu.reset();
    //cpu.set_pc(0x100);
    emulator_loop(cpu, disp, sdl);

    println!("Exiting ...");
}

fn emulator_loop(mut cpu: cpu::Cpu, mut disp: display::Display, sdl: Sdl) {
    'outer: loop {
        cpu.step();
        cpu.mem.step();
        cpu.mem.reg_if |= cpu.mem.timer.step(cpu.cycle);
        cpu.mem.reg_if |= cpu.mem.video.step(cpu.cycle);
        cpu.mem.joypad.step();

        if cpu.mem.video.image_ready {
            // Display the picture!
            disp.render_screen(&cpu.mem.video.screen);

            while let Some(ev) = sdl.event_pump().unwrap().poll_event() {
                match ev {
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'outer,
                    Event::Quit { .. } => break 'outer,
                    _ => {}
                }
                if let Some((button, pressed)) = decode_keyboard(&ev) {
                    cpu.mem.joypad.set_button(button, pressed);
                }
                if let Some((button, pressed)) = decode_gamecontroller(&ev) {
                    cpu.mem.joypad.set_button(button, pressed);
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

fn decode_keyboard(ev: &Event) -> Option<(joypad::JoypadButton, bool)> {
    return match *ev {
        Event::KeyDown { keycode: Some(Keycode::Return), .. } => Some((joypad::JoypadButton::Start, true)),
        Event::KeyDown { keycode: Some(Keycode::Up), .. } => Some((joypad::JoypadButton::Up, true)),
        Event::KeyDown { keycode: Some(Keycode::Down), .. } => Some((joypad::JoypadButton::Down, true)),
        Event::KeyDown { keycode: Some(Keycode::Left), .. } => Some((joypad::JoypadButton::Left, true)),
        Event::KeyDown { keycode: Some(Keycode::Right), .. } => Some((joypad::JoypadButton::Right, true)),
        Event::KeyDown { keycode: Some(Keycode::S), .. } => Some((joypad::JoypadButton::A, true)),
        Event::KeyDown { keycode: Some(Keycode::A), .. } => Some((joypad::JoypadButton::B, true)),
        Event::KeyUp { keycode: Some(Keycode::Return), .. } => Some((joypad::JoypadButton::Start, false)),
        Event::KeyUp { keycode: Some(Keycode::Up), .. } => Some((joypad::JoypadButton::Up, false)),
        Event::KeyUp { keycode: Some(Keycode::Down), .. } => Some((joypad::JoypadButton::Down, false)),
        Event::KeyUp { keycode: Some(Keycode::Left), .. } => Some((joypad::JoypadButton::Left, false)),
        Event::KeyUp { keycode: Some(Keycode::Right), .. } => Some((joypad::JoypadButton::Right, false)),
        Event::KeyUp { keycode: Some(Keycode::S), .. } => Some((joypad::JoypadButton::A, false)),
        Event::KeyUp { keycode: Some(Keycode::A), .. } => Some((joypad::JoypadButton::B, false)),
        _ => None
    }
}

fn decode_gamecontroller(ev: &Event) -> Option<(joypad::JoypadButton, bool)> {
    return match *ev {
        Event::ControllerButtonDown { button: Button::Start, .. } => Some((joypad::JoypadButton::Start, true)),
        Event::ControllerButtonUp { button: Button::Start, .. } => Some((joypad::JoypadButton::Start, false)),
        Event::ControllerButtonDown { button: Button::Back, .. } => Some((joypad::JoypadButton::Select, true)),
        Event::ControllerButtonUp { button: Button::Back, .. } => Some((joypad::JoypadButton::Select, false)),

        Event::ControllerButtonDown { button: Button::A, .. } => Some((joypad::JoypadButton::B, true)),
        Event::ControllerButtonUp { button: Button::A, .. } => Some((joypad::JoypadButton::B, false)),
        Event::ControllerButtonDown { button: Button::B, .. } => Some((joypad::JoypadButton::A, true)),
        Event::ControllerButtonUp { button: Button::B, .. } => Some((joypad::JoypadButton::A, false)),

        Event::ControllerButtonDown { button: Button::DPadUp, .. } => Some((joypad::JoypadButton::Up, true)),
        Event::ControllerButtonUp { button: Button::DPadUp, .. } => Some((joypad::JoypadButton::Up, false)),
        Event::ControllerButtonDown { button: Button::DPadDown, .. } => Some((joypad::JoypadButton::Down, true)),
        Event::ControllerButtonUp { button: Button::DPadDown, .. } => Some((joypad::JoypadButton::Down, false)),
        Event::ControllerButtonDown { button: Button::DPadLeft, .. } => Some((joypad::JoypadButton::Left, true)),
        Event::ControllerButtonUp { button: Button::DPadLeft, .. } => Some((joypad::JoypadButton::Left, false)),
        Event::ControllerButtonDown { button: Button::DPadRight, .. } => Some((joypad::JoypadButton::Right, true)),
        Event::ControllerButtonUp { button: Button::DPadRight, .. } => Some((joypad::JoypadButton::Right, false)),
        _ => None,
    }
}
