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
            0x03 => self.inc16(Reg16::BC),
            0x04 => self.inc8(Reg8::B),
            0x05 => self.dec8(Reg8::B),
            0x06 => {
                // TODO LD B,d8
                self.reg_b = self.read_byte(pc + 1);
                self.reg_pc += 1;
                self.last_m = 2;
                self.last_t = 8;
                println!("LD B, {:#X}", self.reg_b);
            }
            0x09 => self.add_HL(Reg16::BC),
            0x0B => self.dec16(Reg16::BC),
            0x0C => self.inc8(Reg8::C),
            0x0D => self.dec8(Reg8::C),
            0x0E => {
                // TODO LD C,d8
                self.reg_c = self.read_byte(pc + 1);
                self.reg_pc += 1;
                self.last_m = 2;
                self.last_t = 8;
                println!("LD C, {:#X}", self.reg_c);
            }
            0x13 => self.inc16(Reg16::DE),
            0x14 => self.inc8(Reg8::D),
            0x16 => self.dec8(Reg8::D),
            0x19 => self.add_HL(Reg16::DE),
            0x1B => self.dec16(Reg16::DE),
            0x1C => self.inc8(Reg8::E),
            0x1D => self.dec8(Reg8::E),
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

            0x23 => self.inc16(Reg16::HL),
            0x24 => self.inc8(Reg8::H),
            0x25 => self.dec8(Reg8::H),
            0x29 => self.add_HL(Reg16::HL),
            0x2A => {
                // TODO LD A,(HL+)
                println!("LD A, (HL+)");
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let value = self.read_byte(addr);
                self.reg_a = value;
                self.reg_l += 1;
                if self.reg_l == 0x00 {
                    self.reg_h += 1;
                }
                self.last_m = 2;
                self.last_t = 8;
            }
            0x2B => self.dec16(Reg16::HL),
            0x2C => self.inc8(Reg8::L),
            0x2D => self.dec8(Reg8::L),
            0x2F => {
                println!("CPL");
                self.reg_a = self.reg_a ^ 0xFF;
                self.flag_sub = true;
                self.flag_half = true;
                self.last_m = 1;
                self.last_t = 4;
            }
            0x31 => {
                // TODO LD SP,d16
                let value = self.read_word(pc + 1);
                self.reg_pc += 2;
                self.reg_sp = value;
                self.last_m = 3;
                self.last_t = 12;
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

            0x33 => self.inc16(Reg16::SP),
            0x34 => self.inc8(Reg8::AtHL),
            0x35 => self.dec8(Reg8::AtHL),
            0x36 => {
                // TODO LD (HL),d8
                let value = self.read_byte(pc + 1);
                self.reg_pc += 1;
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.write_byte(value, addr);
                self.last_m = 3;
                self.last_t = 12;
                println!("LD (HL), {:#X}", value);
            }
            0x39 => self.add_HL(Reg16::SP),
            0x3B => self.dec16(Reg16::SP),
            0x3C => self.inc8(Reg8::A),
            0x3D => self.dec8(Reg8::A),
            0x3E => {
                // TODO LD A,d8
                self.reg_a = self.read_byte(pc + 1);
                self.reg_pc += 1;
                self.last_m = 2;
                self.last_t = 8;
                println!("LD A, {:#X}", self.reg_a);
            }
            0x47 => {
                // TODO LD B, A
                println!("LD B, A");
                self.reg_b = self.reg_a;
                self.last_m = 1;
                self.last_t = 4;
            }
            0x4F => {
                // TODO LD C, A
                println!("LD C, A");
                self.reg_c = self.reg_a;
                self.last_m = 1;
                self.last_t = 4;
            }
            0x5F => {
                // TODO LD E, A
                println!("LD E, A");
                self.reg_e = self.reg_a;
                self.last_m = 1;
                self.last_t = 4;
            }
            0x78 => {
                // TODO LD A, B
                println!("LD A, B");
                self.reg_a = self.reg_b;
                self.last_m = 1;
                self.last_t = 4;
            }
            0x79 => {
                // TODO LD A, C
                println!("LD A, C");
                self.reg_a = self.reg_c;
                self.last_m = 1;
                self.last_t = 4;
            }
            0x80 => self.add(Reg8::B),
            0x81 => self.add(Reg8::C),
            0x82 => self.add(Reg8::D),
            0x83 => self.add(Reg8::E),
            0x84 => self.add(Reg8::H),
            0x85 => self.add(Reg8::L),
            0x86 => self.add(Reg8::AtHL),
            0x87 => self.add(Reg8::A),

            0x88 => self.adc(Reg8::B),
            0x89 => self.adc(Reg8::C),
            0x8A => self.adc(Reg8::D),
            0x8B => self.adc(Reg8::E),
            0x8C => self.adc(Reg8::H),
            0x8D => self.adc(Reg8::L),
            0x8E => self.adc(Reg8::AtHL),
            0x8F => self.adc(Reg8::A),

            0x90 => self.sub(Reg8::B),
            0x91 => self.sub(Reg8::C),
            0x92 => self.sub(Reg8::D),
            0x93 => self.sub(Reg8::E),
            0x94 => self.sub(Reg8::H),
            0x95 => self.sub(Reg8::L),
            0x96 => self.sub(Reg8::AtHL),
            0x97 => self.sub(Reg8::A),

            0x98 => self.sbc(Reg8::B),
            0x99 => self.sbc(Reg8::C),
            0x9A => self.sbc(Reg8::D),
            0x9B => self.sbc(Reg8::E),
            0x9C => self.sbc(Reg8::H),
            0x9D => self.sbc(Reg8::L),
            0x9E => self.sbc(Reg8::AtHL),
            0x9F => self.sbc(Reg8::A),

            0xA0 => self.and(Reg8::B),
            0xA1 => self.and(Reg8::C),
            0xA2 => self.and(Reg8::D),
            0xA3 => self.and(Reg8::E),
            0xA4 => self.and(Reg8::H),
            0xA5 => self.and(Reg8::L),
            0xA6 => self.and(Reg8::AtHL),
            0xA7 => self.and(Reg8::A),

            0xA8 => self.xor(Reg8::B),
            0xA9 => self.xor(Reg8::C),
            0xAA => self.xor(Reg8::D),
            0xAB => self.xor(Reg8::E),
            0xAC => self.xor(Reg8::H),
            0xAD => self.xor(Reg8::L),
            0xAE => self.xor(Reg8::AtHL),
            0xAF => self.xor(Reg8::A),

            0xB0 => self.or(Reg8::B),
            0xB1 => self.or(Reg8::C),
            0xB2 => self.or(Reg8::D),
            0xB3 => self.or(Reg8::E),
            0xB4 => self.or(Reg8::H),
            0xB5 => self.or(Reg8::L),
            0xB6 => self.or(Reg8::AtHL),
            0xB7 => self.or(Reg8::A),

            0xB8 => self.cp(Reg8::B),
            0xB9 => self.cp(Reg8::C),
            0xBA => self.cp(Reg8::D),
            0xBB => self.cp(Reg8::E),
            0xBC => self.cp(Reg8::H),
            0xBD => self.cp(Reg8::L),
            0xBE => self.cp(Reg8::AtHL),
            0xBF => self.cp(Reg8::A),

            0xC3 => {
                // TODO JP a16
                let old_pc = self.reg_pc;
                self.reg_pc = self.read_word(old_pc);
                self.last_m = 4;
                self.last_t = 16;
                println!("JP {:#X}", self.reg_pc);
            }

            0xC6 => self.addi(false),
            0xC9 => {
                // TODO RET
                println!("RET");
                let sp = self.reg_sp;
                let addr = self.read_word(sp);
                self.reg_pc = addr;
                self.reg_sp = sp + 2;
                self.last_m = 4;
                self.last_t = 16;
            }
            0xCB => {
                // TODO 0xCB instructions
                let op = self.read_byte(pc + 1);
                self.reg_pc += 1;
                match op {
                    0x37 => {
                        // TODO SWAP A
                        println!("SWAP A");
                        let lo = self.reg_a & 0x0F;
                        let hi = self.reg_a & 0xF0;
                        self.reg_a = lo << 4 | hi >> 4;
                        self.flag_zero = self.reg_a == 0x00;
                        self.flag_sub = false;
                        self.flag_half = false;
                        self.flag_carry = false;
                        self.last_m = 2;
                        self.last_t = 8;
                    }
                    _ => panic!("Unknown CB op: {:#X} at addr: {:#X}", op, pc)
                }
            }
            0xCD => {
                // TODO CALL a16
                let addr = self.read_word(pc + 1);
                println!("CALL {:#X}", addr);
                self.reg_sp -= 2;
                let sp = self.reg_sp;
                self.write_word(pc + 3, sp);
                self.reg_pc = addr;
                self.last_m = 6;
                self.last_t = 24;
            }
            0xCE => self.addi(true),
            0xEA => {
                // TODO LD (a16),A
                let addr = self.read_word(pc + 1);
                self.reg_pc += 2;
                let value = self.reg_a;
                self.write_byte(value, addr);
                self.last_m = 4;
                self.last_t = 16;
                println!("LD ({:#X}), A", addr);
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
            0xE1 => {
                // TODO POP HL
                println!("POP HL");
                let sp = self.reg_sp;
                let hl = self.read_word(sp);
                self.reg_h = (hl >> 8) as u8;
                self.reg_l = (hl & 0xFF) as u8;
                self.reg_sp += 2;
                self.last_m = 3;
                self.last_t = 12;
            }
            0xE2 => {
                // TODO LD (C),A
                println!("LD (0xFF00 + C), A");
                let value = self.reg_a;
                let addr = (self.reg_c as u16) + 0xFF00;
                self.write_byte(value, addr);
                self.last_m = 2;
                self.last_t = 8;
            }
            0xE6 => self.andi(),

            0xEF => {
                // TODO RST 28H
                println!("RST 28H");
                let sp = self.reg_sp - 2;
                self.write_word(pc, sp);
                self.reg_pc = 0x28;
                self.reg_sp = sp;
                self.last_m = 4;
                self.last_t = 16;
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
            0xFB => {
                // TODO Enable Interrupts
                println!("EI");
                self.ime = true;
                self.last_m = 1;
                self.last_t = 4;
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

    fn add(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        let old = self.reg_a;
        let other_reg = match reg {
            Reg8::A => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_a);
                self.reg_a
            },
            Reg8::B => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_b);
                self.reg_b
            },
            Reg8::C => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_c);
                self.reg_c
            },
            Reg8::D => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_d);
                self.reg_d
            },
            Reg8::E => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_e);
                self.reg_e
            },
            Reg8::H => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_h);
                self.reg_h
            },
            Reg8::L => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_l);
                self.reg_l
            },
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let other = self.read_byte(addr);
                self.reg_a = self.reg_a.wrapping_add(other);
                self.last_m = 2;
                self.last_t = 8;
                other
            },
        };
        let value = self.reg_a;
        self.flag_zero = value == 0x00;
        self.flag_sub = false;
        self.flag_half = (old & 0x0F + other_reg & 0x0F) & 0x10 == 0x10;
        self.flag_carry = (old as u16) + (other_reg as u16) > 255;
    }

    fn add_HL(&mut self, reg: Reg16) {
        let old = (self.reg_h as u16) << 8 | self.reg_l as u16;
        let other_reg = match reg {
            Reg16::BC => (self.reg_b as u16) << 8 | self.reg_c as u16,
            Reg16::DE => (self.reg_d as u16) << 8 | self.reg_e as u16,
            Reg16::HL => old,
            Reg16::SP => self.reg_sp,
        };
        let value = old.wrapping_add(other_reg);
        self.reg_h = (value >> 8) as u8;
        self.reg_l = (value & 0xFF) as u8;
        self.flag_sub = false;
        self.flag_half = (old & 0x0F00 + other_reg & 0x0F00) & 0x1000 == 0x1000;
        self.flag_carry = (old as u32) + (other_reg as u32) >= 0x10000;
        self.last_m = 2;
        self.last_t = 8;
    }

    fn addi(&mut self, carry: bool) {
        let old = self.reg_a;
        let addr = self.reg_pc;
        let value = self.read_byte(addr);
        self.reg_pc += 1;
        let mut result = old.wrapping_add(value);
        if carry && self.flag_carry { result = result.wrapping_add(1); };
        self.reg_a = result;
        self.flag_zero = result == 0x00;
        self.flag_sub = false;
        self.flag_half = (old & 0xF + value & 0xF) & 0x10 == 0x10;
        if carry && self.flag_carry {
            self.flag_carry = (old as u16) + (value as u16) + 1 > 255;
        } else {
            self.flag_carry = (old as u16) + (value as u16) > 255;
        }
        self.last_m = 2;
        self.last_t = 8;
    }

    fn adc(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        let old = self.reg_a;
        let other_reg = match reg {
            Reg8::A => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_a);
                self.reg_a
            },
            Reg8::B => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_b);
                self.reg_b
            },
            Reg8::C => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_c);
                self.reg_c
            },
            Reg8::D => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_d);
                self.reg_d
            },
            Reg8::E => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_e);
                self.reg_e
            },
            Reg8::H => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_h);
                self.reg_h
            },
            Reg8::L => {
                self.reg_a = self.reg_a.wrapping_add(self.reg_l);
                self.reg_l
            },
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let other = self.read_byte(addr);
                self.reg_a = self.reg_a.wrapping_add(other);
                self.last_m = 2;
                self.last_t = 8;
                other
            },
        };
        if self.flag_carry {
            self.reg_a += 1;
        }
        let value = self.reg_a;
        self.flag_zero = value == 0x00;
        self.flag_sub = false;
        self.flag_half = (old & 0x0F + other_reg & 0x0F) & 0x10 == 0x10;
        self.flag_carry = if self.flag_carry {
            (old as u16) + (other_reg as u16) + 1 > 255
        } else {
            (old as u16) + (other_reg as u16) > 255
        }
    }

    fn andi(&mut self) {
        self.last_m = 2;
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = self.read_byte(pc);
        self.reg_pc += 1;
        self.reg_a = self.reg_a & value;
        self.flag_zero = self.reg_a == 0x00;
        self.flag_sub = false;
        self.flag_half = true;
        self.flag_carry = false;
    }

    fn and(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        match reg {
            Reg8::A => self.reg_a = self.reg_a & self.reg_a,
            Reg8::B => self.reg_a = self.reg_a & self.reg_b,
            Reg8::C => self.reg_a = self.reg_a & self.reg_c,
            Reg8::D => self.reg_a = self.reg_a & self.reg_d,
            Reg8::E => self.reg_a = self.reg_a & self.reg_e,
            Reg8::H => self.reg_a = self.reg_a & self.reg_h,
            Reg8::L => self.reg_a = self.reg_a & self.reg_l,
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.reg_a = self.reg_a & self.read_byte(addr);
                self.last_m = 2;
                self.last_t = 8;
            },
        };
        self.flag_zero = self.reg_a == 0x00;
        self.flag_sub = false;
        self.flag_half = true;
        self.flag_carry = false;
    }

    fn cp(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        let old = self.reg_a;
        let value;
        let result = match reg {
            Reg8::A => {
                value = self.reg_a;
                self.reg_a.wrapping_sub(self.reg_a)
            },
            Reg8::B => {
                value = self.reg_b;
                self.reg_a.wrapping_sub(self.reg_b)
            },
            Reg8::C => {
                value = self.reg_c;
                self.reg_a.wrapping_sub(self.reg_c)
            },
            Reg8::D => {
                value = self.reg_d;
                self.reg_a.wrapping_sub(self.reg_d)
            },
            Reg8::E => {
                value = self.reg_e;
                self.reg_a.wrapping_sub(self.reg_e)
            },
            Reg8::H => {
                value = self.reg_h;
                self.reg_a.wrapping_sub(self.reg_h)
            },
            Reg8::L => {
                value = self.reg_l;
                self.reg_a.wrapping_sub(self.reg_l)
            },
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                value = self.read_byte(addr);
                self.last_m = 2;
                self.last_t = 8;
                self.reg_a.wrapping_sub(value)
            },
        };
        self.flag_zero = result == 0x00;
        self.flag_sub = true;
        self.flag_half = old & 0x0F < value & 0x0F;
        self.flag_carry = old < value;
    }

    fn dec8(&mut self, reg: Reg8) {
        let old;
        let value;
        self.last_m = 1;
        self.last_t = 4;
        match reg {
            Reg8::A => {
                old = self.reg_a;
                value = old.wrapping_sub(1);
                self.reg_a = value;
            },
            Reg8::B => {
                old = self.reg_b;
                value = old.wrapping_sub(1);
                self.reg_b = value;
            },
            Reg8::C => {
                old = self.reg_c;
                value = old.wrapping_sub(1);
                self.reg_c = value;
            },
            Reg8::D => {
                old = self.reg_d;
                value = old.wrapping_sub(1);
                self.reg_d = value;
            },
            Reg8::E => {
                old = self.reg_e;
                value = old.wrapping_sub(1);
                self.reg_e = value;
            },
            Reg8::H => {
                old = self.reg_h;
                value = old.wrapping_sub(1);
                self.reg_h = value;
            },
            Reg8::L => {
                old = self.reg_l;
                value = old.wrapping_sub(1);
                self.reg_l = value;
            },
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                old = self.read_byte(addr);
                value = old.wrapping_sub(1);
                self.write_byte(value, addr);
                self.last_m = 3;
                self.last_t = 12;
            }
        }
        self.flag_zero = value == 0x00;
        self.flag_sub = true;
        self.flag_half = old & 0x0F < 0x01;

    }

    fn dec16(&mut self, reg: Reg16) {
        match reg {
            Reg16::BC => {
                let old = (self.reg_b as u16) << 8 | self.reg_c as u16;
                let value = old.wrapping_sub(1);
                self.reg_b = (value >> 8) as u8;
                self.reg_c = value as u8;
            },
            Reg16::DE => {
                let old = (self.reg_d as u16) << 8 | self.reg_e as u16;
                let value = old.wrapping_sub(1);
                self.reg_d = (value >> 8) as u8;
                self.reg_e = value as u8;
            },
            Reg16::HL => {
                let old = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let value = old.wrapping_sub(1);
                self.reg_h = (value >> 8) as u8;
                self.reg_l = value as u8;
            },
            Reg16::SP => {
                self.reg_sp = self.reg_sp.wrapping_sub(1);
            },
        }
        self.last_m = 2;
        self.last_t = 8;
    }

    fn inc8(&mut self, reg: Reg8) {
        let old;
        let value;
        self.last_m = 1;
        self.last_t = 4;
        match reg {
            Reg8::A => {
                old = self.reg_a;
                value = old.wrapping_add(1);
                self.reg_a = value;
            },
            Reg8::B => {
                old = self.reg_b;
                value = old.wrapping_add(1);
                self.reg_b = value;
            },
            Reg8::C => {
                old = self.reg_c;
                value = old.wrapping_add(1);
                self.reg_c = value;
            },
            Reg8::D => {
                old = self.reg_d;
                value = old.wrapping_add(1);
                self.reg_d = value;
            },
            Reg8::E => {
                old = self.reg_e;
                value = old.wrapping_add(1);
                self.reg_e = value;
            },
            Reg8::H => {
                old = self.reg_h;
                value = old.wrapping_add(1);
                self.reg_h = value;
            },
            Reg8::L => {
                old = self.reg_l;
                value = old.wrapping_add(1);
                self.reg_l = value;
            },
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                old = self.read_byte(addr);
                value = old.wrapping_add(1);
                self.write_byte(value, addr);
                self.last_m = 3;
                self.last_t = 12;
            }
        }
        self.flag_zero = value == 0x00;
        self.flag_sub = false;
        self.flag_half = old & 0x0F + 1 == 0x10;

    }

    fn inc16(&mut self, reg: Reg16) {
        match reg {
            Reg16::BC => {
                let old = (self.reg_b as u16) << 8 | self.reg_c as u16;
                let value = old.wrapping_add(1);
                self.reg_b = (value >> 8) as u8;
                self.reg_c = value as u8;
            },
            Reg16::DE => {
                let old = (self.reg_d as u16) << 8 | self.reg_e as u16;
                let value = old.wrapping_add(1);
                self.reg_d = (value >> 8) as u8;
                self.reg_e = value as u8;
            },
            Reg16::HL => {
                let old = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let value = old.wrapping_add(1);
                self.reg_h = (value >> 8) as u8;
                self.reg_l = value as u8;
            },
            Reg16::SP => {
                self.reg_sp = self.reg_sp.wrapping_add(1);
            },
        }
        self.last_m = 2;
        self.last_t = 8;
    }

    fn or(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        match reg {
            Reg8::A => self.reg_a = self.reg_a | self.reg_a,
            Reg8::B => self.reg_a = self.reg_a | self.reg_b,
            Reg8::C => self.reg_a = self.reg_a | self.reg_c,
            Reg8::D => self.reg_a = self.reg_a | self.reg_d,
            Reg8::E => self.reg_a = self.reg_a | self.reg_e,
            Reg8::H => self.reg_a = self.reg_a | self.reg_h,
            Reg8::L => self.reg_a = self.reg_a | self.reg_l,
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.reg_a = self.reg_a | self.read_byte(addr);
                self.last_m = 2;
                self.last_t = 8;
            },
        };
        self.flag_zero = self.reg_a == 0x00;
        self.flag_sub = false;
        self.flag_half = false;
        self.flag_carry = false;
    }

    fn sbc(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        let old = self.reg_a;
        let other_reg = match reg {
            Reg8::A => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_a);
                self.reg_a
            },
            Reg8::B => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_b);
                self.reg_b
            },
            Reg8::C => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_c);
                self.reg_c
            },
            Reg8::D => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_d);
                self.reg_d
            },
            Reg8::E => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_e);
                self.reg_e
            },
            Reg8::H => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_h);
                self.reg_h
            },
            Reg8::L => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_l);
                self.reg_l
            },
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let other = self.read_byte(addr);
                self.reg_a = self.reg_a.wrapping_sub(other);
                self.last_m = 2;
                self.last_t = 8;
                other
            },
        };
        if self.flag_carry {
            self.reg_a -= 1;
        }
        let value = self.reg_a;
        self.flag_zero = value == 0x00;
        self.flag_sub = true;
        self.flag_half = old & 0x0F < other_reg & 0x0F;
        self.flag_carry = old < other_reg;
    }

    fn sub(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        let old = self.reg_a;
        let other_reg = match reg {
            Reg8::A => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_a);
                self.reg_a
            },
            Reg8::B => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_b);
                self.reg_b
            },
            Reg8::C => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_c);
                self.reg_c
            },
            Reg8::D => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_d);
                self.reg_d
            },
            Reg8::E => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_e);
                self.reg_e
            },
            Reg8::H => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_h);
                self.reg_h
            },
            Reg8::L => {
                self.reg_a = self.reg_a.wrapping_sub(self.reg_l);
                self.reg_l
            },
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let other = self.read_byte(addr);
                self.reg_a = self.reg_a.wrapping_sub(other);
                self.last_m = 2;
                self.last_t = 8;
                other
            },
        };
        let value = self.reg_a;
        self.flag_zero = value == 0x00;
        self.flag_sub = true;
        self.flag_half = old & 0x0F < other_reg & 0x0F;
        self.flag_carry = old < other_reg;
    }

    fn xor(&mut self, reg: Reg8) {
        self.last_m = 1;
        self.last_t = 4;
        match reg {
            Reg8::A => self.reg_a = self.reg_a ^ self.reg_a,
            Reg8::B => self.reg_a = self.reg_a ^ self.reg_b,
            Reg8::C => self.reg_a = self.reg_a ^ self.reg_c,
            Reg8::D => self.reg_a = self.reg_a ^ self.reg_d,
            Reg8::E => self.reg_a = self.reg_a ^ self.reg_e,
            Reg8::H => self.reg_a = self.reg_a ^ self.reg_h,
            Reg8::L => self.reg_a = self.reg_a ^ self.reg_l,
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.reg_a = self.reg_a ^ self.read_byte(addr);
                self.last_m = 2;
                self.last_t = 8;
            },
        };
        self.flag_zero = self.reg_a == 0x00;
        self.flag_sub = false;
        self.flag_half = false;
        self.flag_carry = false;
    }

    fn read_byte(&mut self, addr: u16) -> u8 {
        self.mmu.read_byte(addr as usize)
    }

    fn read_word(&mut self, addr: u16) -> u16 {
        self.mmu.read_word(addr as usize)
    }

    fn write_byte(&mut self, value: u8, addr: u16) {
        self.mmu.write_byte(value, addr as usize);
    }

    fn write_word(&mut self, value: u16, addr: u16) {
        self.mmu.write_word(value, addr as usize);
    }
}

enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    AtHL,
}

enum Reg16 {
    BC,
    DE,
    HL,
    SP,
}
