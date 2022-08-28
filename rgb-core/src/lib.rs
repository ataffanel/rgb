#![warn(clippy::all)]
#![allow(clippy::upper_case_acronyms)]

pub mod cart;
pub mod cpu;
pub mod mem;
pub mod video;
pub mod bootstrap;
pub mod joypad;
pub mod timer;
pub mod audio;

mod dmg;

pub use dmg::Dmg;