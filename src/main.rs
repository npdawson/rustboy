extern crate byteorder;

mod cpu;
mod dmg;
mod mmu;

use std::fs;
use std::env;
use std::io::Read;
use std::path::Path;

fn main() {
    let rom_file_name = env::args().nth(1).unwrap();

    let rom = read_bin(rom_file_name);

    let mut dmg = dmg::Dmg::new(rom);
    //dmg.run();
    // dmg.step();
    loop {
        println!("Current State: {:#?}", dmg);
        dmg.step();
    }
    // println!("After: {:#?}", dmg);
//    println!("{:#?}", &dmg);
}

fn read_bin<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let mut file = fs::File::open(path).unwrap();
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).unwrap();
    file_buf
}
