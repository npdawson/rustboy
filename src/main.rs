extern crate byteorder;
extern crate sdl2;
#[macro_use]
extern crate nom;

#[macro_use]
mod events;
mod dmg;
mod debugger;

use std::fs;
use std::env;
use std::io::Read;
use std::path::Path;
use sdl2::pixels::PixelFormatEnum;

struct_events!{
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down
    },
    else: {
        quit: Quit { .. }
    }
}

fn main() {
    // Init SDL2
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    // Create window
    let window = video.window("Rustboy", 320, 288)
        .position_centered().opengl()
        .build().unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build().unwrap();
    let mut texture = renderer.create_texture_streaming(
        PixelFormatEnum::RGB24, 160, 144).unwrap();

    let boot_file_name = env::args().nth(1).unwrap();
    let rom_file_name = env::args().nth(2).unwrap();

    let boot = read_bin(boot_file_name);
    let rom = read_bin(rom_file_name);

    let mut dmg = dmg::Dmg::new(boot, rom);

    let mut events = Events::new(sdl_context.event_pump().unwrap());

    loop {
        events.pump();

        if events.now.quit || events.now.key_escape == Some(true) {
            break;
        }

        dmg.step();

        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for i in 0..(160 * 144) {
                let offset = i * 3;
                buffer[offset] = dmg.framebuffer()[i].red();
                buffer[offset + 1] = dmg.framebuffer()[i].green();
                buffer[offset + 2] = dmg.framebuffer()[i].blue();
            }
        }).unwrap();

        renderer.clear();
        renderer.copy(&texture, None, None);
        renderer.present();
    }
}

fn read_bin<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf.into_boxed_slice()
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    Off,
    Light,
    Dark,
    On
}

impl Color {
    pub fn from_u8(value: u8) -> Color {
        use self::Color::*;
        match value {
            1 => Light,
            2 => Dark,
            3 => On,
            _ => Off
        }
    }

    pub fn red(&self) -> u8 {
        match *self {
            Color::Off => 156,
            Color::Light => 140,
            Color::Dark => 48,
            Color::On => 15
        }
    }

    pub fn green(&self) -> u8 {
        match *self {
            Color::Off => 189,
            Color::Light => 173,
            Color::Dark => 98,
            Color::On => 56
        }
    }

    pub fn blue(&self) -> u8 {
        match *self {
            Color::Off => 15,
            Color::Light => 15,
            Color::Dark => 48,
            Color::On => 15
        }
    }
 }
