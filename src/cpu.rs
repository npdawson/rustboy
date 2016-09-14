use mmu;

use std::fmt::Debug;

#[derive(Debug)]
pub struct Cpu {
    reg_pc: u16,
    reg_sp: u16,
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    // flags
    flag_zero: bool,
    flag_sub: bool,
    flag_half: bool,
    flag_carry: bool,
    // Interrupt Master Enable
    ime: bool,
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
            reg_b: 0x00,
            reg_c: 0x13,
            reg_d: 0x00,
            reg_e: 0xD8,
            reg_h: 0x01,
            reg_l: 0x4D,
            // flags
            flag_zero: true,
            flag_sub: false,
            flag_half: true,
            flag_carry: true,
            // Interrupt Master Enable
            ime: true,
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
            self.step();
        }
    }

    pub fn step(&mut self) {
        let pc = self.reg_pc;
        self.reg_pc += 1;
        let opcode = self.read_byte(pc);

        println!("PC: {:#x}", pc);

        match opcode {
            0x00 => {
                // NOP
                self.last_m = 1;
                self.last_t = 4;
                println!("NOP");
            },
            0x01 => {
                //TODO LD BC,d16
                self.reg_c = self.read_byte(pc + 1);
                self.reg_b = self.read_byte(pc + 2);
                self.reg_pc += 2;
                self.last_m = 3;
                self.last_t = 12;
                println!("LD BC, {:#X}{:X}", self.reg_b, self.reg_c);
            },
            0x02 => {
                //TODO LD (BC),A
                let value = self.reg_a;
                let addr = (self.reg_b as u16) << 8 | self.reg_c as u16;
                self.write_byte(value, addr);
                self.last_m = 2;
                self.last_t = 8;
                println!("LD (BC), A");
            },
            0x03 => {
                //TODO INC BC
                self.reg_c += 1;
                if self.reg_c == 0x00 {
                    self.reg_b += 1;
                }
                self.last_m = 2;
                self.last_t = 8;
                println!("INC BC");
            },
            0x04 => {
                //TODO INC B
                let old_b = self.reg_b;
                self.reg_b += 1;
                self.flag_zero = self.reg_b == 0x00;
                self.flag_sub = false;
                self.flag_half = (old_b & 0x0F + 1) & 0xF0 == 0x10;
                self.last_m = 1;
                self.last_t = 4;
                println!("INC B");
            },
            0x05 => {
                //TODO DEC B
                let old_b = self.reg_b;
                self.reg_b = self.reg_b.wrapping_sub(1);
                self.flag_zero = self.reg_b == 0x00;
                self.flag_sub = true;
                self.flag_half = old_b & 0x0F < 0x01;
                self.last_m = 1;
                self.last_t = 4;
                println!("DEC B");
            },
            0x06 => {
                // TODO LD B,d8
                self.reg_b = self.read_byte(pc + 1);
                self.reg_pc += 1;
                self.last_m = 2;
                self.last_t = 8;
                println!("LD B, {:#X}", self.reg_b);
            }

            0x0D => {
                // TODO DEC C
                let old_c = self.reg_c;
                self.reg_c = self.reg_c.wrapping_sub(1);
                self.flag_zero = self.reg_c == 0x00;
                self.flag_sub = true;
                self.flag_half = old_c & 0x0F < 0x01;
                self.last_m = 1;
                self.last_t = 4;
                println!("DEC C");
            }

            0x20 => {
                // TODO JR NZ,r8
                let rel_addr = self.read_byte(pc + 1) as i8;
                self.reg_pc += 1;
                if self.flag_zero {
                    self.last_m = 2;
                    self.last_t = 8;
                } else {
                    self.reg_pc = self.reg_pc.wrapping_add(rel_addr as u16);
                    self.last_m = 3;
                    self.last_t = 12;
                }
                println!("JR NZ, {}", rel_addr);
            }
            0x21 => {
                // TODO LD HL,d16
                self.reg_l = self.read_byte(pc + 1);
                self.reg_h = self.read_byte(pc + 2);
                self.reg_pc += 2;
                self.last_m = 3;
                self.last_t = 12;
                println!("LD HL, {:#X}{:X}", self.reg_h, self.reg_l);
            }

            0x0E => {
                // TODO LD C,d8
                self.reg_c = self.read_byte(pc + 1);
                self.reg_pc += 1;
                self.last_m = 2;
                self.last_t = 8;
                println!("LD C, {:#X}", self.reg_c);
            }

            0x32 => {
                // TODO LD (HL-),A
                let value = self.reg_a;
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.write_byte(value, addr);
                self.reg_l = self.reg_l.wrapping_sub(1);
                if self.reg_l == 0xFF {
                    self.reg_h = self.reg_h.wrapping_sub(1);
                }
                self.last_m = 2;
                self.last_t = 8;
                println!("LD (HL-),A");
            }

            0x3E => {
                // TODO LD A,d8
                self.reg_a = self.read_byte(pc + 1);
                self.reg_pc += 1;
                self.last_m = 2;
                self.last_t = 8;
                println!("LD A, {:#X}", self.reg_a);
            }

            0xAF => {
                // TODO XOR A
                self.reg_a = 0;
                self.flag_zero = false;
                self.flag_sub = false;
                self.flag_half = false;
                self.flag_carry = false;
                self.last_m = 1;
                self.last_t = 4;
                println!("XOR A");
            }

            0xC3 => {
                // TODO JP a16
                self.reg_pc = self.read_byte(pc + 1) as u16 |
                (self.read_byte(pc + 2) as u16) << 8;
                self.last_m = 4;
                self.last_t = 16;
                println!("JP {:#X}", self.reg_pc);
            }

            0xE0 => {
                // TODO LDH (a8),A
                let offset = self.read_byte(pc + 1);
                let addr = 0xFF00 + offset as u16;
                let value = self.reg_a;
                self.reg_pc += 1;
                self.write_byte(value, addr);
                self.last_m = 3;
                self.last_t = 12;
                println!("LDH ({:#X}),A", addr);
            }

            0xF0 => {
                // TODO LDH A,(a8)
                let offset = self.read_byte(pc + 1);
                let addr = 0xFF00 + offset as u16;
                let value = self.read_byte(addr);
                self.reg_pc += 1;
                self.reg_a = value;
                self.last_m = 3;
                self.last_t = 12;
                println!("LDH A,({:#X})", addr);
            }

            0xF3 => {
                // TODO Disable Interrupts
                self.ime = false;
                self.last_m = 1;
                self.last_t = 4;
                println!("DI");
            }

            0xFE => {
                // TODO CP d8
                let value = self.read_byte(pc + 1);
                self.reg_pc += 1;
                let old_a = self.reg_a;
                let result = old_a.wrapping_sub(value);
                self.flag_zero = result == 0x00;
                self.flag_sub = true;
                self.flag_half = old_a & 0x0F < value;
                self.flag_carry = old_a < value;
                self.last_m = 2;
                self.last_t = 8;
                println!("CP {:#X}", value);
            }

            _ => panic!("Unknown opcode: {:#X} at address {:#X}", opcode, pc)
        }

        self.clock_m += self.last_m;
        self.clock_t += self.last_t;
    }

    fn read_byte(&mut self, addr: u16) -> u8 {
        self.mmu.read_byte(addr as usize)
    }

    fn write_byte(&mut self, value: u8, addr: u16) {
        self.mmu.write_byte(value, addr as usize);
    }
}
