extern crate byteorder;
extern crate minifb;

mod cpu;
mod dmg;
mod gpu;
mod mmu;

use std::fs;
use std::env;
use std::io::Read;
use std::path::Path;
use minifb::{Window, WindowOptions};

fn main() {
    let boot_file_name = env::args().nth(1).unwrap();
    let rom_file_name = env::args().nth(2).unwrap();

    let boot = read_bin(boot_file_name);
    let rom = read_bin(rom_file_name);

    let mut dmg = dmg::Dmg::new(boot, rom);
    let mut window = match Window::new("Test", 160, 144, WindowOptions::default()) {
        Ok(win) => win,
        Err(err) => {
            println!("unable to create window {}", err);
            return;
        }
    };
    //dmg.run();
    // dmg.step();
    while window.is_open() {
        window.update();
//        println!("Current State: {:#?}", dmg);
        dmg.step();
    }
    //    println!("{:#?}", &dmg);
}

fn read_bin<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}
