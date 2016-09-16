use mmu;
use self::regs::{Regs,Reg8,Reg16,JF,ID};

mod regs;

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
    pub fn new(boot: Vec<u8>, rom: Vec<u8>) -> Cpu {
        Cpu {
            regs: Regs::default(),
            // Interrupt Master Enable
            ime: true,
            // clock time of last instruction
            last_t: 0,
            // clock time total
            clock_t: 0,
            mmu: mmu::Mmu::new(boot, rom),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let pc = self.regs.pc;
        if pc == 0xFFFF { panic!("wild!"); }
        let opcode = self.read_byte(pc);
        self.regs.pc = pc.saturating_add(1);

        println!("PC: {:#x}", pc);

        self.op(opcode);

        let cycles = self.last_t;
        self.clock_t += cycles;
        self.mmu.step_gpu(cycles);

    }


    fn add(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.add(reg);
    }

    fn add_to_HL(&mut self, reg: Reg16) {
        self.last_t = 8;
        self.regs.add_to_HL(reg);
    }

    fn add_HL(&mut self) {
        self.last_t = 8;
        let addr = self.regs.read16(Reg16::HL);
        let value = self.read_byte(addr);
        self.regs.add_HL(value);
    }

    fn addi(&mut self, carry: bool) {
        let addr = self.regs.pc;
        let value = self.read_byte(addr);
        self.regs.pc += 1;
        self.regs.addi(value, carry);
        self.last_t = 8;
    }

    fn adc(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.adc(reg);
    }

    fn andi(&mut self) {
        self.last_t = 8;
        let pc = self.regs.pc;
        let value = self.read_byte(pc);
        self.regs.pc += 1;
        self.regs.andi(value);
    }

    fn and(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.and(reg);
    }

    fn bit(&mut self, bit: u8, reg: Reg8) {
        self.last_t = 8;
        self.regs.bit(bit, reg);
    }

    fn cp(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.cp(reg);
    }

    fn cp_HL(&mut self) {
        self.last_t = 8;
        let addr = self.regs.read16(Reg16::HL);
        let value = self.read_byte(addr);
        self.regs.cp_HL(value);
    }

    fn dec8(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.dec(reg);
    }

    fn dec16(&mut self, reg: Reg16) {
        self.last_t = 8;
        self.regs.dec16(reg);
    }

    fn inc8(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.inc(reg);
    }

    fn inc16(&mut self, reg: Reg16) {
        self.last_t = 8;
        self.regs.inc16(reg);
    }

    fn jump_match(&self, flag:JF) -> bool {
        self.regs.jump_match(flag)
    }

    fn jr(&mut self, flag: JF) {
        let pc = self.regs.pc;
        let rel_addr = self.read_byte(pc) as i8;
        self.regs.pc = pc.saturating_add(1);
        let jump = self.jump_match(flag);
        let jump_addr = self.regs.pc.wrapping_add(rel_addr as u16);
        if jump {
            self.regs.pc = jump_addr;
            self.last_t = 12;
        } else {
            self.last_t = 8;
        }
    }

    fn jp(&mut self, flag: JF) {
        let pc = self.regs.pc;
        let addr = self.read_word(pc);
        self.regs.pc += 2;
        let jump = self.jump_match(flag);
        if jump {
            self.regs.pc = addr;
            self.last_t = 16;
        } else {
            self.last_t = 12;
        }
    }

    fn ld(&mut self, rd: Reg8, rs: Reg8) { // rs : source ; rd : destination
        self.last_t = 4;
        self.regs.ld(rd, rs);
    }

    fn ldi(&mut self, reg: Reg8) {
        self.last_t = 8;
        let pc = self.regs.pc;
        let value = self.read_byte(pc);
        self.regs.pc += 1;
        self.regs.write(value, reg);
    }

    fn ldi16(&mut self, reg: Reg16) {
        self.last_t = 12;
        let pc = self.regs.pc;
        let value = self.read_word(pc);
        self.regs.pc += 2;
        self.regs.write16(value, reg);
    }

    fn ld_from_HL(&mut self, reg: Reg8) {
        self.last_t = 8;
        let addr = self.regs.read16(Reg16::HL);
        let value = self.read_byte(addr);
        self.regs.write(value, reg);
    }

    fn ld_to_HL(&mut self, reg: Reg8) {
        self.last_t = 8;
        let addr = self.regs.read16(Reg16::HL);
        let value = self.regs.read(reg);
        self.write_byte(value, addr);
    }

    fn ld_from_mem(&mut self, reg: Reg16, id: ID) {
        let addr = self.regs.read16(reg);
        let next_addr = match id {
            ID::None => addr,
            ID::Inc => addr + 1,
            ID::Dec => addr - 1,
        };
        self.regs.write16(next_addr, reg);
        let value = self.read_byte(addr);
        self.regs.write(value, Reg8::A);
        self.last_t = 8;
    }

    fn ld_to_mem(&mut self, reg: Reg16, id: ID) {
        let value = self.regs.read(Reg8::A);
        let addr = self.regs.read16(reg);
        let next_addr = match id {
            ID::None => addr,
            ID::Inc => addr + 1,
            ID::Dec => addr - 1,
        };
        self.regs.write16(next_addr, reg);
        self.write_byte(value, addr);
        self.last_t = 8;
    }

    fn or(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.or(reg);
    }

    fn pop(&mut self, reg: Reg16) {
        let sp = self.regs.sp;
        let value = self.read_word(sp);
        self.regs.write16(value, reg);
        self.regs.sp += 2;
        self.last_t = 12;
    }

    fn push(&mut self, reg: Reg16) {
        let sp = self.regs.sp - 2;
        let value = self.regs.read16(reg);
        self.write_word(value, sp);
        self.regs.sp = sp;
        self.last_t = 16;
    }

    fn res(&mut self, bit: u8, reg: Reg8) {
        self.last_t = 8;
        self.regs.res(bit, reg);
    }

    fn ret(&mut self, flag: JF) {
        let jump = self.jump_match(flag);
        if jump {
            let sp = self.regs.sp;
            let addr = self.read_word(sp);
            self.regs.pc = addr;
            self.regs.sp = sp + 2;
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

    fn rl(&mut self, reg: Reg8) {
        self.last_t = 8;
        self.regs.rl(reg);
    }

    fn rla(&mut self) {
        self.last_t = 4;
        self.regs.rla();
    }

    fn rst(&mut self, addr: u16) {
        let sp = self.regs.sp - 2;
        let pc = self.regs.pc;
        self.write_word(pc, sp);
        self.regs.write16(addr, Reg16::PC);
        self.regs.write16(sp, Reg16::SP);
        self.last_t = 16;
    }

    fn sbc(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.sbc(reg);
    }

    fn sub(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.sub(reg);
    }

    fn sub_HL(&mut self) {
        self.last_t = 8;
        let addr = self.regs.read16(Reg16::HL);
        let value = self.read_byte(addr);
        self.regs.sub_HL(value);
    }

    fn swap(&mut self, reg: Reg8) {
        self.last_t = 8;
        self.regs.swap(reg);
    }

    fn xor(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.regs.xor(reg);
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

    fn op(&mut self, opcode: u8) {
        match opcode {
            0x00 => {
                // NOP
                self.last_t = 4;
                println!("NOP");
            },
            0x01 => self.ldi16(Reg16::BC),
            0x02 => self.ld_to_mem(Reg16::BC, ID::None),
            0x03 => self.inc16(Reg16::BC),
            0x04 => self.inc8(Reg8::B),
            0x05 => self.dec8(Reg8::B),
            0x06 => self.ldi(Reg8::B),

            0x09 => self.add_to_HL(Reg16::BC),
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
            0x17 => self.rla(),

            0x18 => self.jr(JF::Always),
            0x19 => self.add_to_HL(Reg16::DE),
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
            0x29 => self.add_to_HL(Reg16::HL),
            0x2A => self.ld_from_mem(Reg16::HL, ID::Inc),
            0x2B => self.dec16(Reg16::HL),
            0x2C => self.inc8(Reg8::L),
            0x2D => self.dec8(Reg8::L),
            0x2E => self.ldi(Reg8::L),
            0x2F => {
                self.last_t = 4;
                self.regs.cpl();
            }

            0x30 => self.jr(JF::NC),
            0x31 => self.ldi16(Reg16::SP),
            0x32 => self.ld_to_mem(Reg16::HL, ID::Dec),
            0x33 => self.inc16(Reg16::SP),
            // 0x34 => self.inc8(Reg8::AtHL),
            // 0x35 => self.dec8(Reg8::AtHL),
            0x36 => { // LD (HL), d8
                self.last_t = 12;
                let pc = self.regs.pc;
                let value = self.read_byte(pc);
                self.regs.pc += 1;
                let addr = self.regs.read16(Reg16::HL);
                self.write_byte(value, addr);
            }

            0x38 => self.jr(JF::C),
            0x39 => self.add_to_HL(Reg16::SP),
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
            0x46 => self.ld_from_HL(Reg8::B),
            0x47 => self.ld(Reg8::B, Reg8::A),

            0x48 => self.ld(Reg8::C, Reg8::B),
            0x49 => self.ld(Reg8::C, Reg8::C),
            0x4A => self.ld(Reg8::C, Reg8::D),
            0x4B => self.ld(Reg8::C, Reg8::E),
            0x4C => self.ld(Reg8::C, Reg8::H),
            0x4D => self.ld(Reg8::C, Reg8::L),
            0x4E => self.ld_from_HL(Reg8::C),
            0x4F => self.ld(Reg8::C, Reg8::A),

            0x50 => self.ld(Reg8::D, Reg8::B),
            0x51 => self.ld(Reg8::D, Reg8::C),
            0x52 => self.ld(Reg8::D, Reg8::D),
            0x53 => self.ld(Reg8::D, Reg8::E),
            0x54 => self.ld(Reg8::D, Reg8::H),
            0x55 => self.ld(Reg8::D, Reg8::L),
            0x56 => self.ld_from_HL(Reg8::D),
            0x57 => self.ld(Reg8::D, Reg8::A),

            0x58 => self.ld(Reg8::E, Reg8::B),
            0x59 => self.ld(Reg8::E, Reg8::C),
            0x5A => self.ld(Reg8::E, Reg8::D),
            0x5B => self.ld(Reg8::E, Reg8::E),
            0x5C => self.ld(Reg8::E, Reg8::H),
            0x5D => self.ld(Reg8::E, Reg8::L),
            0x5E => self.ld_from_HL(Reg8::E),
            0x5F => self.ld(Reg8::E, Reg8::A),

            0x60 => self.ld(Reg8::H, Reg8::B),
            0x61 => self.ld(Reg8::H, Reg8::C),
            0x62 => self.ld(Reg8::H, Reg8::D),
            0x63 => self.ld(Reg8::H, Reg8::E),
            0x64 => self.ld(Reg8::H, Reg8::H),
            0x65 => self.ld(Reg8::H, Reg8::L),
            0x66 => self.ld_from_HL(Reg8::H),
            0x67 => self.ld(Reg8::H, Reg8::A),

            0x68 => self.ld(Reg8::L, Reg8::B),
            0x69 => self.ld(Reg8::L, Reg8::C),
            0x6A => self.ld(Reg8::L, Reg8::D),
            0x6B => self.ld(Reg8::L, Reg8::E),
            0x6C => self.ld(Reg8::L, Reg8::H),
            0x6D => self.ld(Reg8::L, Reg8::L),
            0x6E => self.ld_from_HL(Reg8::L),
            0x6F => self.ld(Reg8::L, Reg8::A),

            0x70 => self.ld_to_HL(Reg8::B),
            0x71 => self.ld_to_HL(Reg8::C),
            0x72 => self.ld_to_HL(Reg8::D),
            0x73 => self.ld_to_HL(Reg8::E),
            0x74 => self.ld_to_HL(Reg8::H),
            0x75 => self.ld_to_HL(Reg8::L),

            0x77 => self.ld_to_HL(Reg8::A),

            0x78 => self.ld(Reg8::A, Reg8::B),
            0x79 => self.ld(Reg8::A, Reg8::C),
            0x7A => self.ld(Reg8::A, Reg8::D),
            0x7B => self.ld(Reg8::A, Reg8::E),
            0x7C => self.ld(Reg8::A, Reg8::H),
            0x7D => self.ld(Reg8::A, Reg8::L),
            0x7E => self.ld_from_HL(Reg8::A),
            0x7F => self.ld(Reg8::A, Reg8::A),

            0x80 => self.add(Reg8::B),
            0x81 => self.add(Reg8::C),
            0x82 => self.add(Reg8::D),
            0x83 => self.add(Reg8::E),
            0x84 => self.add(Reg8::H),
            0x85 => self.add(Reg8::L),
            0x86 => self.add_HL(),
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
            0x96 => self.sub_HL(),
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
            0xBE => self.cp_HL(),
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
                let pc = self.regs.pc;
                let op = self.read_byte(pc);
                self.regs.pc = pc.saturating_add(1);
                match op {
                    0x11 => self.rl(Reg8::C),
                    0x30 => self.swap(Reg8::B),
                    0x37 => self.swap(Reg8::A),
                    0x70 => self.bit(0, Reg8::B),
                    0x7C => self.bit(7, Reg8::H),
                    0x80 => self.res(0, Reg8::B),
                    0x81 => self.res(0, Reg8::C),
                    0x82 => self.res(0, Reg8::D),
                    0x83 => self.res(0, Reg8::E),
                    0x84 => self.res(0, Reg8::H),
                    0x85 => self.res(0, Reg8::L),
//                    0x86 => self.res(0, Reg8::B),
                    0x87 => self.res(0, Reg8::A),
                    0x88 => self.res(1, Reg8::B),
                    _ => panic!("Unknown CB op: {:#X} at addr: {:#X}", op, pc - 1),
                }
            },
            0xCD => {
                // TODO CALL a16
                let pc = self.regs.pc;
                let addr = self.read_word(pc);
                self.regs.sp -= 2;
                let sp = self.regs.sp;
                self.write_word(pc + 2, sp);
                self.regs.pc = addr;
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
                let pc = self.regs.pc;
                let offset = self.read_byte(pc);
                let addr = 0xFF00 + offset as u16;
                let value = self.regs.read(Reg8::A);
                self.regs.pc += 1;
                self.write_byte(value, addr);
                self.last_t = 12;
                println!("LDH ({:#X}),A", addr);
            }
            0xE1 => self.pop(Reg16::HL),
            0xE2 => {
                // TODO LD (C),A
                println!("LD (0xFF00 + C), A");
                let value = self.regs.read(Reg8::A);
                let addr = (self.regs.read(Reg8::C) as u16) + 0xFF00;
                self.write_byte(value, addr);
                self.last_t = 8;
            }
            0xE5 => self.push(Reg16::HL),
            0xE6 => self.andi(),
            0xE7 => self.rst(0x20),
            0xE9 => {
                // TODO JP (HL)
                println!("JP (HL)");
                let addr = self.regs.read16(Reg16::HL);
                self.regs.pc = addr;
                self.last_t = 4;
            }
            0xEA => {
                // TODO LD (a16),A
                let pc = self.regs.pc;
                let addr = self.read_word(pc);
                self.regs.pc += 2;
                let value = self.regs.read(Reg8::A);
                self.write_byte(value, addr);
                self.last_t = 16;
                println!("LD ({:#X}), A", addr);
            }
            0xEF => self.rst(0x28),
            0xF0 => {
                // TODO LDH A,(a8)
                let pc = self.regs.pc;
                let offset = self.read_byte(pc);
                let addr = 0xFF00 + offset as u16;
                let value = self.read_byte(addr);
                self.regs.pc += 1;
                self.regs.write(value, Reg8::A);
                self.last_t = 12;
                println!("LDH A,({:#X})", addr);
            }
            0xF1 => self.pop(Reg16::AF),
            0xF3 => {
                // Disable Interrupts
                self.ime = false;
                self.last_t = 4;
            }
            0xF5 => self.push(Reg16::AF),
            0xF7 => self.rst(0x30),
            0xFA => { // LD A, (a16)
                let pc = self.regs.pc;
                let addr = self.read_word(pc);
                self.regs.pc += 2;
                let value = self.read_byte(addr);
                self.regs.write(value, Reg8::A);
                self.last_t = 12;
            }
            0xFB => {
                // Enable Interrupts
                self.ime = true;
                self.last_t = 4;
            }

            0xFE => { // CP d8
                self.last_t = 8;
                let pc = self.regs.pc;
                let value = self.read_byte(pc);
                self.regs.pc += 1;
                self.regs.cpi(value);
            }
            // 0xFF => self.rst(0x38),

            _ => panic!("Unknown opcode: {:#X} at address {:#X}", opcode, self.regs.pc - 1)
        }
    }
}
