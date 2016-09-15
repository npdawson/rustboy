mod regs;

use mmu;

#[derive(Debug)]
pub struct Cpu {
    regs: Regs,
    // Interrupt Master Enable
    ime: bool,
    // clock time of last instruction
    last_t: usize,
    // clock time total
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
            last_t: 0,
            // clock time total
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
                // let next_op = self.read_byte(pc + 1);
                // if next_op == 0x00 {
                //     panic!("gah!");
                // }
                self.last_t = 4;
                println!("NOP");
            },
            0x01 => self.ldi16(Reg16::BC),
            0x02 => self.ld_to_mem(Reg16::BC, ID::None),
            0x03 => self.inc16(Reg16::BC),
            0x04 => self.inc8(Reg8::B),
            0x05 => self.dec8(Reg8::B),
            0x06 => self.ldi(Reg8::B),

            0x09 => self.add_HL(Reg16::BC),
            0x0A => self.ld_from_mem(Reg16::BC, ID::None),
            0x0B => self.dec16(Reg16::BC),
            0x0C => self.inc8(Reg8::C),
            0x0D => self.dec8(Reg8::C),
            0x0E => self.ldi(Reg8::C),

            0x11 => self.ldi16(Reg16::DE),
            0x12 => self.ld_to_mem(Reg16::DE, ID::None),
            0x13 => self.inc16(Reg16::DE),
            0x14 => self.inc8(Reg8::D),
            0x15 => self.dec8(Reg8::D),
            0x16 => self.ldi(Reg8::D),

            0x18 => self.jr(JF::Always),
            0x19 => self.add_HL(Reg16::DE),
            0x1A => self.ld_from_mem(Reg16::DE, ID::None),
            0x1B => self.dec16(Reg16::DE),
            0x1C => self.inc8(Reg8::E),
            0x1D => self.dec8(Reg8::E),
            0x1E => self.ldi(Reg8::E),

            0x20 => self.jr(JF::NZ),
            0x21 => self.ldi16(Reg16::HL),
            0x22 => self.ld_to_mem(Reg16::HL, ID::Inc),
            0x23 => self.inc16(Reg16::HL),
            0x24 => self.inc8(Reg8::H),
            0x25 => self.dec8(Reg8::H),
            0x26 => self.ldi(Reg8::H),

            0x28 => self.jr(JF::Z),
            0x29 => self.add_HL(Reg16::HL),
            0x2A => self.ld_from_mem(Reg16::HL, ID::Inc),
            0x2B => self.dec16(Reg16::HL),
            0x2C => self.inc8(Reg8::L),
            0x2D => self.dec8(Reg8::L),
            0x2E => self.ldi(Reg8::L),
            0x2F => {
                println!("CPL");
                self.reg_a = self.reg_a ^ 0xFF;
                self.flag_sub = true;
                self.flag_half = true;
                self.last_t = 4;
            }

            0x30 => self.jr(JF::NC),
            0x31 => self.ldi16(Reg16::SP),
            0x32 => self.ld_to_mem(Reg16::HL, ID::Dec),
            0x33 => self.inc16(Reg16::SP),
            // 0x34 => self.inc8(Reg8::AtHL),
            // 0x35 => self.dec8(Reg8::AtHL),
            // 0x36 => self.ldi(Reg8::AtHL),

            0x38 => self.jr(JF::C),
            0x39 => self.add_HL(Reg16::SP),
            0x3A => self.ld_from_mem(Reg16::HL, ID::Dec),
            0x3B => self.dec16(Reg16::SP),
            0x3C => self.inc8(Reg8::A),
            0x3D => self.dec8(Reg8::A),
            0x3E => self.ldi(Reg8::A),

            0x40 => self.ld(Reg8::B, Reg8::B),
            0x41 => self.ld(Reg8::B, Reg8::C),
            0x42 => self.ld(Reg8::B, Reg8::D),
            0x43 => self.ld(Reg8::B, Reg8::E),
            0x44 => self.ld(Reg8::B, Reg8::H),
            0x45 => self.ld(Reg8::B, Reg8::L),
            // 0x46 => self.ld(Reg8::B, Reg8::AtHL),
            0x47 => self.ld(Reg8::B, Reg8::A),

            0x48 => self.ld(Reg8::C, Reg8::B),
            0x49 => self.ld(Reg8::C, Reg8::C),
            0x4A => self.ld(Reg8::C, Reg8::D),
            0x4B => self.ld(Reg8::C, Reg8::E),
            0x4C => self.ld(Reg8::C, Reg8::H),
            0x4D => self.ld(Reg8::C, Reg8::L),
            // 0x4E => self.ld(Reg8::C, Reg8::AtHL),
            0x4F => self.ld(Reg8::C, Reg8::A),

            0x50 => self.ld(Reg8::D, Reg8::B),
            0x51 => self.ld(Reg8::D, Reg8::C),
            0x52 => self.ld(Reg8::D, Reg8::D),
            0x53 => self.ld(Reg8::D, Reg8::E),
            0x54 => self.ld(Reg8::D, Reg8::H),
            0x55 => self.ld(Reg8::D, Reg8::L),
            // 0x56 => self.ld(Reg8::D, Reg8::AtHL),
            0x57 => self.ld(Reg8::D, Reg8::A),

            0x58 => self.ld(Reg8::E, Reg8::B),
            0x59 => self.ld(Reg8::E, Reg8::C),
            0x5A => self.ld(Reg8::E, Reg8::D),
            0x5B => self.ld(Reg8::E, Reg8::E),
            0x5C => self.ld(Reg8::E, Reg8::H),
            0x5D => self.ld(Reg8::E, Reg8::L),
            // 0x5E => self.ld(Reg8::E, Reg8::AtHL),
            0x5F => self.ld(Reg8::E, Reg8::A),

            0x60 => self.ld(Reg8::H, Reg8::B),
            0x61 => self.ld(Reg8::H, Reg8::C),
            0x62 => self.ld(Reg8::H, Reg8::D),
            0x63 => self.ld(Reg8::H, Reg8::E),
            0x64 => self.ld(Reg8::H, Reg8::H),
            0x65 => self.ld(Reg8::H, Reg8::L),
            // 0x66 => self.ld(Reg8::H, Reg8::AtHL),
            0x67 => self.ld(Reg8::H, Reg8::A),

            0x68 => self.ld(Reg8::L, Reg8::B),
            0x69 => self.ld(Reg8::L, Reg8::C),
            0x6A => self.ld(Reg8::L, Reg8::D),
            0x6B => self.ld(Reg8::L, Reg8::E),
            0x6C => self.ld(Reg8::L, Reg8::H),
            0x6D => self.ld(Reg8::L, Reg8::L),
            // 0x6E => self.ld(Reg8::L, Reg8::AtHL),
            0x6F => self.ld(Reg8::L, Reg8::A),

            // 0x70 => self.ld(Reg8::AtHL, Reg8::B),
            // 0x71 => self.ld(Reg8::AtHL, Reg8::C),
            // 0x72 => self.ld(Reg8::AtHL, Reg8::D),
            // 0x73 => self.ld(Reg8::AtHL, Reg8::E),
            // 0x74 => self.ld(Reg8::AtHL, Reg8::H),
            // 0x75 => self.ld(Reg8::AtHL, Reg8::L),
            // 0x77 => self.ld(Reg8::AtHL, Reg8::A),

            0x78 => self.ld(Reg8::A, Reg8::B),
            0x79 => self.ld(Reg8::A, Reg8::C),
            0x7A => self.ld(Reg8::A, Reg8::D),
            0x7B => self.ld(Reg8::A, Reg8::E),
            0x7C => self.ld(Reg8::A, Reg8::H),
            0x7D => self.ld(Reg8::A, Reg8::L),
            // 0x7E => self.ld(Reg8::A, Reg8::AtHL),
            0x7F => self.ld(Reg8::A, Reg8::A),

            0x80 => self.add(Reg8::B),
            0x81 => self.add(Reg8::C),
            0x82 => self.add(Reg8::D),
            0x83 => self.add(Reg8::E),
            0x84 => self.add(Reg8::H),
            0x85 => self.add(Reg8::L),
            // 0x86 => self.add(Reg8::AtHL),
            0x87 => self.add(Reg8::A),

            0x88 => self.adc(Reg8::B),
            0x89 => self.adc(Reg8::C),
            0x8A => self.adc(Reg8::D),
            0x8B => self.adc(Reg8::E),
            0x8C => self.adc(Reg8::H),
            0x8D => self.adc(Reg8::L),
            // 0x8E => self.adc(Reg8::AtHL),
            0x8F => self.adc(Reg8::A),

            0x90 => self.sub(Reg8::B),
            0x91 => self.sub(Reg8::C),
            0x92 => self.sub(Reg8::D),
            0x93 => self.sub(Reg8::E),
            0x94 => self.sub(Reg8::H),
            0x95 => self.sub(Reg8::L),
            // 0x96 => self.sub(Reg8::AtHL),
            0x97 => self.sub(Reg8::A),

            0x98 => self.sbc(Reg8::B),
            0x99 => self.sbc(Reg8::C),
            0x9A => self.sbc(Reg8::D),
            0x9B => self.sbc(Reg8::E),
            0x9C => self.sbc(Reg8::H),
            0x9D => self.sbc(Reg8::L),
            // 0x9E => self.sbc(Reg8::AtHL),
            0x9F => self.sbc(Reg8::A),

            0xA0 => self.and(Reg8::B),
            0xA1 => self.and(Reg8::C),
            0xA2 => self.and(Reg8::D),
            0xA3 => self.and(Reg8::E),
            0xA4 => self.and(Reg8::H),
            0xA5 => self.and(Reg8::L),
            // 0xA6 => self.and(Reg8::AtHL),
            0xA7 => self.and(Reg8::A),

            0xA8 => self.xor(Reg8::B),
            0xA9 => self.xor(Reg8::C),
            0xAA => self.xor(Reg8::D),
            0xAB => self.xor(Reg8::E),
            0xAC => self.xor(Reg8::H),
            0xAD => self.xor(Reg8::L),
            // 0xAE => self.xor(Reg8::AtHL),
            0xAF => self.xor(Reg8::A),

            0xB0 => self.or(Reg8::B),
            0xB1 => self.or(Reg8::C),
            0xB2 => self.or(Reg8::D),
            0xB3 => self.or(Reg8::E),
            0xB4 => self.or(Reg8::H),
            0xB5 => self.or(Reg8::L),
            // 0xB6 => self.or(Reg8::AtHL),
            0xB7 => self.or(Reg8::A),

            0xB8 => self.cp(Reg8::B),
            0xB9 => self.cp(Reg8::C),
            0xBA => self.cp(Reg8::D),
            0xBB => self.cp(Reg8::E),
            0xBC => self.cp(Reg8::H),
            0xBD => self.cp(Reg8::L),
            // 0xBE => self.cp(Reg8::AtHL),
            0xBF => self.cp(Reg8::A),

            0xC0 => self.ret(JF::NZ),
            0xC1 => self.pop(Reg16::BC),
            0xC2 => self.jp(JF::NZ),
            0xC3 => self.jp(JF::Always),
            0xC5 => self.push(Reg16::BC),
            0xC6 => self.addi(false),
            0xC7 => self.rst(0x00),

            0xC8 => self.ret(JF::Z),
            0xC9 => self.ret(JF::Always),
            0xCA => self.jp(JF::Z),
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
                self.last_t = 24;
            }
            0xCE => self.addi(true),
            0xCF => self.rst(0x08),

            0xD0 => self.ret(JF::NC),
            0xD1 => self.pop(Reg16::DE),
            0xD2 => self.jp(JF::NC),
            0xD5 => self.push(Reg16::DE),
            0xD7 => self.rst(0x10),

            0xD8 => self.ret(JF::C),
            0xD9 => {
                self.ret(JF::Always);
                self.ime = true;
            }
            0xDA => self.jp(JF::C),
            0xDF => self.rst(0x18),

            0xE0 => {
                // TODO LDH (a8),A
                let offset = self.read_byte(pc + 1);
                let addr = 0xFF00 + offset as u16;
                let value = self.reg_a;
                self.reg_pc += 1;
                self.write_byte(value, addr);
                self.last_t = 12;
                println!("LDH ({:#X}),A", addr);
            }
            0xE1 => self.pop(Reg16::HL),
            0xE2 => {
                // TODO LD (C),A
                println!("LD (0xFF00 + C), A");
                let value = self.reg_a;
                let addr = (self.reg_c as u16) + 0xFF00;
                self.write_byte(value, addr);
                self.last_t = 8;
            }
            0xE5 => self.push(Reg16::HL),
            0xE6 => self.andi(),
            0xE7 => self.rst(0x20),
            0xE9 => {
                // TODO JP (HL)
                println!("JP (HL)");
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.reg_pc = addr;
                self.last_t = 4;
            }
            0xEA => {
                // TODO LD (a16),A
                let addr = self.read_word(pc + 1);
                self.reg_pc += 2;
                let value = self.reg_a;
                self.write_byte(value, addr);
                self.last_t = 16;
                println!("LD ({:#X}), A", addr);
            }
            0xEF => self.rst(0x28),
            0xF0 => {
                // TODO LDH A,(a8)
                let offset = self.read_byte(pc + 1);
                let addr = 0xFF00 + offset as u16;
                let value = self.read_byte(addr);
                self.reg_pc += 1;
                self.reg_a = value;
                self.last_t = 12;
                println!("LDH A,({:#X})", addr);
            }
            0xF1 => self.pop(Reg16::AF),
            0xF3 => {
                // TODO Disable Interrupts
                self.ime = false;
                self.last_t = 4;
                println!("DI");
            }
            0xF5 => self.push(Reg16::AF),
            0xF7 => self.rst(0x30),
            0xFA => {
                let pc = self.reg_pc;
                let addr = self.read_word(pc);
                self.reg_pc += 2;
                self.reg_a = self.read_byte(addr);
                self.last_t = 12;
            }
            0xFB => {
                // TODO Enable Interrupts
                println!("EI");
                self.ime = true;
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
                self.last_t = 8;
                println!("CP {:#X}", value);
            }
           // 0xFF => self.rst(0x38),

            _ => panic!("Unknown opcode: {:#X} at address {:#X}", opcode, pc)
        }

        self.clock_t += self.last_t;
    }

    fn add(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.add(reg);
    }

    fn add_HL(&mut self, reg: Reg16) {
        self.last_t = 8;
        self.regs.add_HL(reg);
    }

    fn addi(&mut self, carry: bool) {
        let addr = self.reg_pc;
        let value = self.read_byte(addr);
        self.reg_pc += 1;
        self.regs.addi(value, carry);
        self.last_t = 8;
    }

    fn adc(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.adc(reg);
    }

    fn andi(&mut self) {
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
                self.last_t = 8;
            },
        };
        self.flag_zero = self.reg_a == 0x00;
        self.flag_sub = false;
        self.flag_half = true;
        self.flag_carry = false;
    }

    fn cp(&mut self, reg: Reg8) {
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
            _ => panic!("Can't decrement AF!")
        }
        self.last_t = 8;
    }

    fn inc8(&mut self, reg: Reg8) {
        let old;
        let value;
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
            _ => panic!("Can't increment AF!")
        }
        self.last_t = 8;
    }

    fn jr(&mut self, flag: JF) {
        let pc = self.reg_pc;
        let rel_addr = self.read_byte(pc) as i8;
        self.reg_pc += 1;
        let jump = match flag {
            JF::Always => true,
            JF::Z => self.flag_zero,
            JF::C => self.flag_carry,
            JF::NZ => !self.flag_zero,
            JF::NC => !self.flag_carry,
        };
        let jump_addr = self.reg_pc.wrapping_add(rel_addr as u16);
        if jump {
            self.reg_pc = jump_addr;
            self.last_t = 12;
        } else {
            self.last_t = 8;
        }
    }

    fn jp(&mut self, flag: JF) {
        let pc = self.reg_pc;
        let addr = self.read_word(pc);
        self.reg_pc += 2;
        let jump = match flag {
            JF::Always => true,
            JF::Z => self.flag_zero,
            JF::C => self.flag_carry,
            JF::NZ => !self.flag_zero,
            JF::NC => !self.flag_carry,
        };
        if jump {
            self.reg_pc = addr;
            self.last_t = 16;
        } else {
            self.last_t = 12;
        }
    }

    fn ld(&mut self, rd: Reg8, rs: Reg8) { // rs : source ; rd : destination
        self.last_t = 4;
        let value = match rs {
            Reg8::A => self.reg_a,
            Reg8::B => self.reg_b,
            Reg8::C => self.reg_c,
            Reg8::D => self.reg_d,
            Reg8::E => self.reg_e,
            Reg8::H => self.reg_h,
            Reg8::L => self.reg_l,
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.last_t = 8;
                self.read_byte(addr)
            }
        };
        match rd {
            Reg8::A => self.reg_a = value,
            Reg8::B => self.reg_b = value,
            Reg8::C => self.reg_c = value,
            Reg8::D => self.reg_d = value,
            Reg8::E => self.reg_e = value,
            Reg8::H => self.reg_h = value,
            Reg8::L => self.reg_l = value,
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.write_byte(value, addr);
                self.last_t = 8;
            }
        };
    }

    fn ldi(&mut self, reg: Reg8) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = self.read_byte(pc);
        self.reg_pc += 1;
        match reg {
            Reg8::A => self.reg_a = value,
            Reg8::B => self.reg_b = value,
            Reg8::C => self.reg_c = value,
            Reg8::D => self.reg_d = value,
            Reg8::E => self.reg_e = value,
            Reg8::H => self.reg_h = value,
            Reg8::L => self.reg_l = value,
            Reg8::AtHL => {
                let addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                self.write_byte(value, addr);
                self.last_t = 12;
            }
        }
    }

    fn ldi16(&mut self, reg: Reg16) {
        self.last_t = 12;
        let pc = self.reg_pc;
        let value = self.read_word(pc);
        self.reg_pc += 2;
        match reg {
            Reg16::BC => {
                self.reg_b = (value >> 8) as u8;
                self.reg_c = (value & 0xFF) as u8;
            }
            Reg16::DE => {
                self.reg_d = (value >> 8) as u8;
                self.reg_e = (value & 0xFF) as u8;
            }
            Reg16::HL => {
                self.reg_h = (value >> 8) as u8;
                self.reg_l = (value & 0xFF) as u8;
            }
            Reg16::SP => self.reg_sp = value,
            _ => panic!("Can't load imm16 into AF!")
        }
    }

    fn ld_from_mem(&mut self, reg: Reg16, id: ID) {
        let addr;
        match reg {
            Reg16::BC => addr = (self.reg_b as u16) << 8 | self.reg_c as u16,
            Reg16::DE => addr = (self.reg_d as u16) << 8 | self.reg_e as u16,
            Reg16::HL => {
                addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let next_addr = match id {
                    ID::None => panic!("Use regular ld() function!"),
                    ID::Inc => addr.wrapping_add(1),
                    ID::Dec => addr.wrapping_sub(1),
                };
                self.reg_h = (next_addr >> 8) as u8;
                self.reg_l = (next_addr & 0xFF) as u8;
            },
            _ => panic!("Can't load from address in AF!")
        };
        self.reg_a = self.read_byte(addr);
        self.last_t = 8;
    }

    fn ld_to_mem(&mut self, reg: Reg16, id: ID) {
        let addr;
        match reg {
            Reg16::BC => addr = (self.reg_b as u16) << 8 | self.reg_c as u16,
            Reg16::DE => addr = (self.reg_d as u16) << 8 | self.reg_e as u16,
            Reg16::HL => {
                addr = (self.reg_h as u16) << 8 | self.reg_l as u16;
                let next_addr = match id {
                    ID::None => panic!("Use regular ld() function!"),
                    ID::Inc => addr.wrapping_add(1),
                    ID::Dec => addr.wrapping_sub(1),
                };
                self.reg_h = (next_addr >> 8) as u8;
                self.reg_l = (next_addr & 0xFF) as u8;
            },
            _ => panic!("Can't load from address in AF!")
        };
        let value = self.reg_a;
        self.write_byte(value, addr);
        self.last_t = 8;
    }

    fn or(&mut self, reg: Reg8) {
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
                self.last_t = 8;
            },
        };
        self.flag_zero = self.reg_a == 0x00;
        self.flag_sub = false;
        self.flag_half = false;
        self.flag_carry = false;
    }

    fn pop(&mut self, reg: Reg16) {
        println!("POP");
        let sp = self.reg_sp;
        let value = self.read_word(sp);
        match reg {
            Reg16::BC => {
                self.reg_b = (value >> 8) as u8;
                self.reg_c = (value & 0xFF) as u8;
            }
            Reg16::DE => {
                self.reg_d = (value >> 8) as u8;
                self.reg_e = (value & 0xFF) as u8;
            }
            Reg16::HL => {
                self.reg_h = (value >> 8) as u8;
                self.reg_l = (value & 0xFF) as u8;
            }
            Reg16::AF => {
                self.reg_a = (value >> 8) as u8;
                self.flag_zero  = (value >> 4) & 0b1000 == 0x1000;
                self.flag_sub   = (value >> 4) & 0b0100 == 0x0100;
                self.flag_half  = (value >> 4) & 0b0010 == 0x0010;
                self.flag_carry = (value >> 4) & 0b0001 == 0x0001;
            }
            _ => panic!("No POP SP!")
        }
        self.reg_sp += 2;
        self.last_t = 12;
    }

    fn push(&mut self, reg: Reg16) {
        println!("PUSH");
        let sp = self.reg_sp - 2;
        let value = match reg {
            Reg16::BC => (self.reg_b as u16) << 8 | self.reg_c as u16,
            Reg16::DE => (self.reg_d as u16) << 8 | self.reg_c as u16,
            Reg16::HL => (self.reg_h as u16) << 8 | self.reg_c as u16,
            Reg16::AF => {
                let mut v = (self.reg_a as u16) << 8;
                if self.flag_zero  { v |= 0x80; }
                if self.flag_sub   { v |= 0x40; }
                if self.flag_half  { v |= 0x20; }
                if self.flag_carry { v |= 0x10; }
                v
            }
            _ => panic!("Can't push SP!")
        };
        self.write_word(value, sp);
        self.reg_sp = sp;
        self.last_t = 16;
    }

    fn ret(&mut self, flag: JF) {
        let jump = match flag {
            JF::Always => true,
            JF::Z => self.flag_zero,
            JF::C => self.flag_carry,
            JF::NZ => !self.flag_zero,
            JF::NC => !self.flag_carry,
        };
        if jump {
            let sp = self.reg_sp;
            let addr = self.read_word(sp);
            self.reg_pc = addr;
            self.reg_sp = sp + 2;
            match flag {
                JF::Always => {
                    self.last_t = 16;
                }
                _ => {
                    self.last_t = 20;
                }
            }
        } else {
            self.last_t = 8;
        }
    }

    fn rst(&mut self, addr: u16) {
        let sp = self.reg_sp - 2;
        let pc = self.reg_pc;
        self.write_word(pc, sp);
        self.reg_pc = addr;
        self.reg_sp = sp;
        self.last_t = 16;
    }

    fn sbc(&mut self, reg: Reg8) {
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
