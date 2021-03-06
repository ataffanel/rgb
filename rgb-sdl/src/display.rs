// Display implementation using SDL2
// The video emulation makes all the work to generate a video framebuffer. This file
// handle more specifically displaying the screen using SDL2
use sdl2::pixels::PixelFormatEnum::BGR24;
use sdl2::render::{Renderer, Texture, TextureAccess};
use sdl2::Sdl;

/// Emulated screen width in pixels
const SCREEN_WIDTH: usize = 160;
/// Emulated screen height in pixels
const SCREEN_HEIGHT: usize = 144;
/// Screen texture size in bytes
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 3;


pub struct Display<'a> {
    pub renderer: Box<Renderer<'a>>,
    pub texture: Box<Texture>,
}

impl<'a> Display<'a> {
    pub fn new() -> (Display<'a>, Sdl) {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        sdl.event().unwrap();
        sdl.timer().unwrap();

        let mut window_builder = video.window("rgb",
                                            (SCREEN_WIDTH as usize) as u32,
                                            (SCREEN_HEIGHT as usize) as u32);
        let window = window_builder.position_centered().build().unwrap();

        let renderer = window.renderer().accelerated().present_vsync().build().unwrap();
        let texture = renderer.create_texture(BGR24,
                                              TextureAccess::Streaming,
                                              SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
                                              .unwrap();
        (Display {
            renderer: Box::new(renderer),
            texture: Box::new(texture),
        }, sdl)
    }

    pub fn render_screen(&mut self, screen_buffer: &[u8]) {
        self.texture.update(None, screen_buffer, SCREEN_WIDTH * 3).unwrap();
        self.renderer.clear();
        self.renderer.copy(&self.texture, None, None).expect("Display I/O error");
        self.renderer.present();
    }


}
