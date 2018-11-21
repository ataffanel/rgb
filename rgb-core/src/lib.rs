pub mod cart;
pub mod cpu;
pub mod mem;
pub mod video;
pub mod bootstrap;
pub mod joypad;
pub mod timer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
