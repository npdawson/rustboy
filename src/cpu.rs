use mmu;

use std::fmt::Debug;

pub struct Cpu {
    reg_pc: u16,
    reg_sp: u16,
    reg_a: u8,
    reg_f: u8, // separate flags?
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    // clock time of last instruction
    last_m: usize,
    last_t: usize,
    // clock time total
    clock_m: usize,
    clock_t: usize,
    mmu: mmu::Mmu,
}

impl Cpu {
    pub fn new(rom: Vec<u8>) -> Cpu {
        Cpu {
            reg_pc: 0x0100,
            reg_sp: 0xFFFE,
            reg_a: 0x01,
            reg_f: 0xB0, // separate flags?
            reg_b: 0x00,
            reg_c: 0x13,
            reg_d: 0x00,
            reg_e: 0xD8,
            reg_h: 0x01,
            reg_l: 0x4D,
            // clock time of last instruction
            last_m: 0,
            last_t: 0,
            // clock time total
            clock_m: 0,
            clock_t: 0,
            mmu: mmu::Mmu::new(rom),
        }
    }

    pub fn run(&mut self) {
        loop {
            let pc = self.reg_pc;
            let opcode = self.read_byte(pc);
            panic!("Opcode: {:#X}", opcode);
        }
    }

    fn read_byte(&mut self, addr: u16) -> u8 {
        self.mmu.read_byte(addr as usize)
    }
}
