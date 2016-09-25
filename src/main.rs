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

        renderer.clear();
        renderer.present();
    }
}

fn read_bin<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf.into_boxed_slice()
}
