use dmg::mmu::Mmu;

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
    flag_reg: Flags,
    // Interrupt Master Enable
    pub ime: bool,
    // halted, waiting for interrupt
    pub halted: bool,
    // clock time of last instruction
    last_t: usize,
    // clock time total
    clock_t: usize,
}

impl Cpu {
    pub fn new() -> Cpu {
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
            flag_reg: Flags::default(),
            // Interrupt Master Enable
            ime: true,
            halted: false,
            // clock time of last instruction
            last_t: 0,
            // clock time total
            clock_t: 0,
        }
    }

    pub fn step(&mut self, mmu: &mut Mmu) -> usize {
        if self.halted{
            4 // wait 4 cycles at a time for interrupt
        } else {
            let pc = self.reg_pc;
            // if pc == 0xFFFF { panic!("wild!"); }
            let opcode = mmu.read_byte(pc as usize);
            self.reg_pc = pc.saturating_add(1);

            println!("PC: {:#x}", pc);

            self.execute_instr(opcode, mmu);

            let cycles = self.last_t;
            self.clock_t += cycles;
            cycles
        }
    }

    pub fn interrupt(&mut self, addr: u16, mmu: &mut Mmu) -> usize {
        let cycles = 4;
        self.last_t = cycles;
        self.clock_t += cycles;
        self.push(Reg16::PC, mmu);
        self.reg_pc = addr;
        cycles
    }

    fn arith<F>(&mut self, reg: Reg8, f: F)
        where F: FnOnce(u8, u8) -> u8
    {
        self.last_t = 4;
        let old = self.reg_a;
        let value = self.read_reg(reg);
        let result = f(old, value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
    }

    fn add(&mut self, reg: Reg8) {
        self.last_t = 4;
        let old = self.reg_a;
        let value = self.read_reg(reg);
        let result = old.wrapping_add(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = (old & 0x0F + value & 0x0F) & 0x10 == 0x10;
        self.flag_reg.carry = (old as u16) + (value as u16) > 255;
    }

    fn adc(&mut self, reg: Reg8) {
        self.add(reg);
        if self.flag_reg.carry {
            let old = self.reg_a;
            self.reg_a = old.wrapping_add(1);
            self.flag_reg.zero = old == 0xFF;
            self.flag_reg.sub = false;
            self.flag_reg.half = (old & 0x0F + 1) & 0x10 == 0x10;
            self.flag_reg.carry = (old as u16) + 1 > 255;
        }
    }

    fn add_to_hl(&mut self, reg: Reg16) {
        self.last_t = 8;
        let old = self.read_reg16(Reg16::HL);
        let value = self.read_reg16(reg);
        let result = old.wrapping_add(value);
        self.write_reg16(Reg16::HL, result);
        self.flag_reg.sub = false;
        self.flag_reg.half =
            (old & 0x0FFF).wrapping_add(value & 0x0FFF) & 0x1000 == 0x1000;
        self.flag_reg.carry = (old as u32) + (value as u32) >= 0x10000;
    }

    fn add_hl(&mut self, mmu: &mut Mmu) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(mmu, addr);
        let old = self.reg_a;
        let result = old.wrapping_add(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = (old & 0x0F + value & 0x0F) & 0x10 == 0x10;
        self.flag_reg.carry = (old as u16) + (value as u16) > 255;
    }

    fn addi(&mut self, carry: bool, mmu: &mut Mmu) {
        self.last_t = 8;
        let addr = self.reg_pc;
        let value = self.read_byte(mmu, addr);
        self.reg_pc += 1;
        let old = self.reg_a;
        let result = if carry && self.flag_reg.carry {
            self.flag_reg.carry = (old as u16) + (value as u16) + 1 > 255;
            old.wrapping_add(value).wrapping_add(1)
        } else {
            self.flag_reg.carry = (old as u16) + (value as u16) > 255;
            old.wrapping_add(value)
        };
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = (old & 0x0F + value & 0x0F) & 0x10 == 0x10;
    }

    fn andi(&mut self, mmu: &mut Mmu) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = self.read_byte(mmu, pc);
        self.reg_pc += 1;
        let old = self.reg_a;
        let result = old & value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = true;
        self.flag_reg.carry = false;
    }

    fn and(&mut self, reg: Reg8) {
        self.last_t = 4;
        let value = self.reg_a & self.read_reg(reg);
        self.reg_a = value;
        self.flag_reg.zero = value == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = true;
        self.flag_reg.carry = false;
    }

    fn bit(&mut self, bit: u8, reg: Reg8) {
        self.last_t = 8;
        let value = self.read_reg(reg);
        let result = value & (1 << bit) != 0;
        self.flag_reg.zero = !result;
        self.flag_reg.sub = false;
        self.flag_reg.half = true;
    }

    fn cp(&mut self, reg: Reg8) {
        self.last_t = 4;
        let old = self.reg_a;
        let value = self.read_reg(reg);
        let result = old.wrapping_sub(value);
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value & 0x0F;
        self.flag_reg.carry = old < value;
    }

    fn cpi(&mut self, imm: u8) {
        let old = self.reg_a;
        let result = old.wrapping_sub(imm);
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < imm;
        self.flag_reg.carry = old < imm;
    }

    fn cp_hl(&mut self, mmu: &mut Mmu) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(mmu, addr);
        let old = self.reg_a;
        let result = old.wrapping_sub(value);
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value & 0x0F;
        self.flag_reg.carry = old < value;
    }

    fn dec8(&mut self, reg: Reg8) {
        self.last_t = 4;
        let old = self.read_reg(reg);
        let value = old.wrapping_sub(1);
        self.write_reg(reg, value);
        self.flag_reg.zero = value == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < 0x01;
    }

    fn dec16(&mut self, reg: Reg16) {
        self.last_t = 8;
        let old = self.read_reg16(reg);
        self.write_reg16(reg, old.wrapping_sub(1));
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn inc8(&mut self, reg: Reg8) {
        self.last_t = 4;
        let old = self.read_reg(reg);
        let value = old.wrapping_add(1);
        self.write_reg(reg, value);
        self.flag_reg.zero = value == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = old & 0x0F + 1 == 0x10;
    }

    fn inc16(&mut self, reg: Reg16) {
        self.last_t = 8;
        let old = self.read_reg16(reg);
        self.write_reg16(reg, old.wrapping_add(1));
    }

    fn jump_match(&self, flag:JF) -> bool {
        match flag {
            JF::Always => true,
            JF::Z => self.flag_reg.zero,
            JF::C => self.flag_reg.carry,
            JF::NZ => !self.flag_reg.zero,
            JF::NC => !self.flag_reg.carry,
        }
    }

    fn jr(&mut self, flag: JF, mmu: &mut Mmu) {
        let pc = self.reg_pc;
        let rel_addr = self.read_byte(mmu, pc) as i8;
        self.reg_pc = pc.saturating_add(1);
        let jump = self.jump_match(flag);
        let jump_addr = self.reg_pc.wrapping_add(rel_addr as u16);
        if jump {
            self.reg_pc = jump_addr;
            self.last_t = 12;
        } else {
            self.last_t = 8;
        }
    }

    fn jp(&mut self, flag: JF, mmu: &mut Mmu) {
        let pc = self.reg_pc;
        let addr = self.read_word(mmu, pc);
        self.reg_pc += 2;
        let jump = self.jump_match(flag);
        if jump {
            self.reg_pc = addr;
            self.last_t = 16;
        } else {
            self.last_t = 12;
        }
    }

    fn ld(&mut self, rd: Reg8, rs: Reg8) { // rs : source ; rd : destination
        self.last_t = 4;
        let value = self.read_reg(rs);
        self.write_reg(rd, value);
    }

    fn ldi(&mut self, reg: Reg8, mmu: &mut Mmu) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = self.read_byte(mmu, pc);
        self.reg_pc += 1;
        self.write_reg(reg, value);
    }

    fn ldi16(&mut self, reg: Reg16, mmu: &mut Mmu) {
        self.last_t = 12;
        let pc = self.reg_pc;
        let value = self.read_word(mmu, pc);
        self.reg_pc += 2;
        self.write_reg16(reg, value);
    }

    fn ld_from_hl(&mut self, reg: Reg8, mmu: &mut Mmu) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(mmu, addr);
        self.write_reg(reg, value);
    }

    fn ld_to_hl(&mut self, reg: Reg8, mmu: &mut Mmu) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_reg(reg);
        self.write_byte(mmu, addr, value);
    }

    fn ld_from_mem(&mut self, reg: Reg16, id: ID, mmu: &mut Mmu) {
        let addr = self.read_reg16(reg);
        let next_addr = match id {
            ID::None => addr,
            ID::Inc => addr + 1,
            ID::Dec => addr - 1,
        };
        self.write_reg16(reg, next_addr);
        let value = self.read_byte(mmu, addr);
        self.reg_a = value;
        self.last_t = 8;
    }

    fn ld_to_mem(&mut self, reg: Reg16, id: ID, mmu: &mut Mmu) {
        self.last_t = 8;
        let value = self.reg_a;
        let addr = self.read_reg16(reg);
        let next_addr = match id {
            ID::None => addr,
            ID::Inc => addr + 1,
            ID::Dec => addr - 1,
        };
        self.write_reg16(reg, next_addr);
        self.write_byte(mmu, addr, value);
    }

    fn or(&mut self, reg: Reg8) {
        self.last_t = 4;
        let value = self.reg_a & self.read_reg(reg);
        self.reg_a = value;
        self.flag_reg.zero = value == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn pop(&mut self, reg: Reg16, mmu: &mut Mmu) {
        let sp = self.reg_sp;
        let value = self.read_word(mmu, sp);
        self.write_reg16(reg, value);
        self.reg_sp += 2;
        self.last_t = 12;
    }

    fn push(&mut self, reg: Reg16, mmu: &mut Mmu) {
        let sp = self.reg_sp - 2;
        let value = self.read_reg16(reg);
        self.write_word(mmu, sp, value);
        self.reg_sp = sp;
        self.last_t = 16;
    }

    fn res(&mut self, bit: u8, reg: Reg8) {
        self.last_t = 8;
        let old = self.read_reg(reg);
        let result = old & !(1 << bit);
        self.write_reg(reg, result);
    }

    fn ret(&mut self, flag: JF, mmu: &mut Mmu) {
        let jump = self.jump_match(flag);
        if jump {
            let sp = self.reg_sp;
            let addr = self.read_word(mmu, sp);
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

    fn rl(&mut self, reg: Reg8) {
        self.last_t = 8;
        let carrybit = self.flag_reg.carry;
        let value = self.read_reg(reg);
        let result = value << 1 | (if carrybit { 1 } else { 0 });
        self.write_reg(reg, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = value & (1 << 7) != 0;
    }

    fn rla(&mut self) {
        self.rl(Reg8::A);
        self.last_t = 4;
        self.flag_reg.zero = false;
    }

    fn rr(&mut self, reg: Reg8) {
        self.last_t = 8;
        let carrybit = self.flag_reg.carry;
        let value = self.read_reg(reg);
        let result = value >> 1 | (if carrybit { 1 << 7 } else { 0 });
        self.write_reg(reg, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = value & (1 << 0) != 0;
    }

    fn rst(&mut self, addr: u16, mmu: &mut Mmu) {
        self.last_t = 16;
        let sp = self.reg_sp - 2;
        let pc = self.reg_pc;
        self.write_word(mmu, sp, pc);
        self.reg_pc = addr;
        self.reg_sp = sp;
    }

    fn sbc(&mut self, reg: Reg8) {
        self.last_t = 4;
        self.sub(reg);
        if self.flag_reg.carry {
            let old = self.reg_a;
            let result = old.wrapping_sub(1);
            self.reg_a = result;
            self.flag_reg.zero = result == 0x00;
            self.flag_reg.sub = true;
            self.flag_reg.half = old & 0x0F < 1;
            self.flag_reg.carry = old < 1;
        }
    }

    fn sub(&mut self, reg: Reg8) {
        self.last_t = 4;
        let old = self.reg_a;
        let value = self.read_reg(reg);
        let result = old.wrapping_sub(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value & 0x0F;
        self.flag_reg.carry = old < value;
    }

    fn subi(&mut self, mmu: &mut Mmu) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let imm = self.read_byte(mmu, pc);
        self.reg_pc += 1;
        let old = self.reg_a;
        let result = old.wrapping_sub(imm);
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < imm & 0x0F;
        self.flag_reg.carry = old < imm;
    }

    fn sub_hl(&mut self, mmu: &mut Mmu) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let old = self.reg_a;
        let value = self.read_byte(mmu, addr);
        let result = old.wrapping_sub(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value & 0x0F;
        self.flag_reg.carry = old < value;
    }

    fn swap(&mut self, reg: Reg8) {
        self.last_t = 8;
        let old = self.read_reg(reg);
        let lo = old & 0x0F;
        let hi = old & 0xF0;
        let result = lo << 4 | hi >> 4;
        self.write_reg(reg, result);
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn xor(&mut self, reg: Reg8) {
        self.last_t = 4;
        let value = self.read_reg(reg);
        let result = self.reg_a ^ value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn xor_hl(&mut self, mmu: &mut Mmu) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(mmu, addr);
        let result = self.reg_a ^ value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn read_byte(&mut self, mmu: &mut Mmu, addr: u16) -> u8 {
        mmu.read_byte(addr as usize)
    }

    fn read_word(&mut self, mmu: &mut Mmu, addr: u16) -> u16 {
        mmu.read_word(addr as usize)
    }

    fn write_byte(&mut self, mmu: &mut Mmu, addr: u16, value: u8) {
        mmu.write_byte(addr as usize, value);
    }

    fn write_word(&mut self, mmu: &mut Mmu, addr: u16, value: u16) {
        mmu.write_word(addr as usize, value);
    }

    fn read_reg(&self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.reg_a,
            Reg8::B => self.reg_b,
            Reg8::C => self.reg_c,
            Reg8::D => self.reg_d,
            Reg8::E => self.reg_e,
            Reg8::F => self.flag_reg.into(),
            Reg8::H => self.reg_h,
            Reg8::L => self.reg_l,
        }
    }

    fn read_reg16(&self, reg: Reg16) -> u16 {
        match reg {
            Reg16::BC =>
                (self.read_reg(Reg8::B) as u16) << 8 | self.read_reg(Reg8::C) as u16,
            Reg16::DE =>
                (self.read_reg(Reg8::D) as u16) << 8 | self.read_reg(Reg8::E) as u16,
            Reg16::HL =>
                (self.read_reg(Reg8::H) as u16) << 8 | self.read_reg(Reg8::L) as u16,
            Reg16::SP => self.reg_sp,
            Reg16::PC => self.reg_pc,
            Reg16::AF =>
                (self.read_reg(Reg8::A) as u16) << 8 | self.flag_reg.into() as u16,
        }
    }

    fn write_reg(&mut self, reg: Reg8, value: u8) {
        match reg {
            Reg8::A => self.reg_a = value,
            Reg8::B => self.reg_b = value,
            Reg8::C => self.reg_c = value,
            Reg8::D => self.reg_d = value,
            Reg8::E => self.reg_e = value,
            Reg8::F => self.flag_reg = value.into(),
            Reg8::H => self.reg_h = value,
            Reg8::L => self.reg_l = value,
        }
    }

    fn write_reg16(&mut self, reg: Reg16, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        match reg {
            Reg16::BC => {
                self.write_reg(Reg8::B, hi);
                self.write_reg(Reg8::C, lo);
            }
            Reg16::DE => {
                self.write_reg(Reg8::D, hi);
                self.write_reg(Reg8::E, lo);
            }
            Reg16::HL => {
                self.write_reg(Reg8::H, hi);
                self.write_reg(Reg8::L, lo);
            }
            Reg16::SP => self.reg_sp = value,
            Reg16::PC => self.reg_pc = value,
            Reg16::AF => {
                self.write_reg(Reg8::A, hi);
                self.write_reg(Reg8::F, lo);
            }
        }
    }

    fn execute_instr(&mut self, opcode: u8, mmu: &mut Mmu) {
        match opcode {
            0x00 => {
                // NOP
                self.last_t = 4;
                println!("NOP");
            },
            0x01 => self.ldi16(Reg16::BC, mmu),
            0x02 => self.ld_to_mem(Reg16::BC, ID::None, mmu),
            0x03 => self.inc16(Reg16::BC),
            0x04 => self.inc8(Reg8::B),
            0x05 => self.dec8(Reg8::B),
            0x06 => self.ldi(Reg8::B, mmu),

            0x09 => self.add_to_hl(Reg16::BC),
            0x0A => self.ld_from_mem(Reg16::BC, ID::None, mmu),
            0x0B => self.dec16(Reg16::BC),
            0x0C => self.inc8(Reg8::C),
            0x0D => self.dec8(Reg8::C),
            0x0E => self.ldi(Reg8::C, mmu),

            0x11 => self.ldi16(Reg16::DE, mmu),
            0x12 => self.ld_to_mem(Reg16::DE, ID::None, mmu),
            0x13 => self.inc16(Reg16::DE),
            0x14 => self.inc8(Reg8::D),
            0x15 => self.dec8(Reg8::D),
            0x16 => self.ldi(Reg8::D, mmu),
            0x17 => self.rla(),

            0x18 => self.jr(JF::Always, mmu),
            0x19 => self.add_to_hl(Reg16::DE),
            0x1A => self.ld_from_mem(Reg16::DE, ID::None, mmu),
            0x1B => self.dec16(Reg16::DE),
            0x1C => self.inc8(Reg8::E),
            0x1D => self.dec8(Reg8::E),
            0x1E => self.ldi(Reg8::E, mmu),

            0x20 => self.jr(JF::NZ, mmu),
            0x21 => self.ldi16(Reg16::HL, mmu),
            0x22 => self.ld_to_mem(Reg16::HL, ID::Inc, mmu),
            0x23 => self.inc16(Reg16::HL),
            0x24 => self.inc8(Reg8::H),
            0x25 => self.dec8(Reg8::H),
            0x26 => self.ldi(Reg8::H, mmu),

            0x28 => self.jr(JF::Z, mmu),
            0x29 => self.add_to_hl(Reg16::HL),
            0x2A => self.ld_from_mem(Reg16::HL, ID::Inc, mmu),
            0x2B => self.dec16(Reg16::HL),
            0x2C => self.inc8(Reg8::L),
            0x2D => self.dec8(Reg8::L),
            0x2E => self.ldi(Reg8::L, mmu),
            0x2F => {
                self.last_t = 4;
                self.reg_a ^= 0xFF;
                self.flag_reg.sub = true;
                self.flag_reg.half = true;
            }

            0x30 => self.jr(JF::NC, mmu),
            0x31 => self.ldi16(Reg16::SP, mmu),
            0x32 => self.ld_to_mem(Reg16::HL, ID::Dec, mmu),
            0x33 => self.inc16(Reg16::SP),
            0x34 => {
                // inc (HL)
                self.last_t = 12;
                let value = self.read_reg16(Reg16::HL);
                let result = value.wrapping_add(1);
                self.write_reg16(Reg16::HL, result);
                self.flag_reg.zero = result == 0;
                self.flag_reg.sub = false;
                self.flag_reg.half =
                    (value & 0x0FFF).wrapping_add(1) & 0x1000 == 0x1000;
            }
            0x35 => {
                // dec (HL)
                self.last_t = 12;
                let value = self.read_reg16(Reg16::HL);
                let result = value.wrapping_sub(1);
                self.write_reg16(Reg16::HL, result);
                self.flag_reg.zero = result == 0;
                self.flag_reg.sub = true;
                self.flag_reg.half = (value & 0x0FFF) < 1;
            }
            0x36 => { // LD (HL), d8
                self.last_t = 12;
                let pc = self.reg_pc;
                let value = self.read_byte(mmu, pc);
                self.reg_pc += 1;
                let addr = self.read_reg16(Reg16::HL);
                self.write_byte(mmu, addr, value);
            }
            0x37 => {
                // SCF
                self.flag_reg.carry = true;
                self.flag_reg.sub = false;
                self.flag_reg.half = false;
                self.last_t = 4;
            }

            0x38 => self.jr(JF::C, mmu),
            0x39 => self.add_to_hl(Reg16::SP),
            0x3A => self.ld_from_mem(Reg16::HL, ID::Dec, mmu),
            0x3B => self.dec16(Reg16::SP),
            0x3C => self.inc8(Reg8::A),
            0x3D => self.dec8(Reg8::A),
            0x3E => self.ldi(Reg8::A, mmu),

            0x40 => self.ld(Reg8::B, Reg8::B),
            0x41 => self.ld(Reg8::B, Reg8::C),
            0x42 => self.ld(Reg8::B, Reg8::D),
            0x43 => self.ld(Reg8::B, Reg8::E),
            0x44 => self.ld(Reg8::B, Reg8::H),
            0x45 => self.ld(Reg8::B, Reg8::L),
            0x46 => self.ld_from_hl(Reg8::B, mmu),
            0x47 => self.ld(Reg8::B, Reg8::A),

            0x48 => self.ld(Reg8::C, Reg8::B),
            0x49 => self.ld(Reg8::C, Reg8::C),
            0x4A => self.ld(Reg8::C, Reg8::D),
            0x4B => self.ld(Reg8::C, Reg8::E),
            0x4C => self.ld(Reg8::C, Reg8::H),
            0x4D => self.ld(Reg8::C, Reg8::L),
            0x4E => self.ld_from_hl(Reg8::C, mmu),
            0x4F => self.ld(Reg8::C, Reg8::A),

            0x50 => self.ld(Reg8::D, Reg8::B),
            0x51 => self.ld(Reg8::D, Reg8::C),
            0x52 => self.ld(Reg8::D, Reg8::D),
            0x53 => self.ld(Reg8::D, Reg8::E),
            0x54 => self.ld(Reg8::D, Reg8::H),
            0x55 => self.ld(Reg8::D, Reg8::L),
            0x56 => self.ld_from_hl(Reg8::D, mmu),
            0x57 => self.ld(Reg8::D, Reg8::A),

            0x58 => self.ld(Reg8::E, Reg8::B),
            0x59 => self.ld(Reg8::E, Reg8::C),
            0x5A => self.ld(Reg8::E, Reg8::D),
            0x5B => self.ld(Reg8::E, Reg8::E),
            0x5C => self.ld(Reg8::E, Reg8::H),
            0x5D => self.ld(Reg8::E, Reg8::L),
            0x5E => self.ld_from_hl(Reg8::E, mmu),
            0x5F => self.ld(Reg8::E, Reg8::A),

            0x60 => self.ld(Reg8::H, Reg8::B),
            0x61 => self.ld(Reg8::H, Reg8::C),
            0x62 => self.ld(Reg8::H, Reg8::D),
            0x63 => self.ld(Reg8::H, Reg8::E),
            0x64 => self.ld(Reg8::H, Reg8::H),
            0x65 => self.ld(Reg8::H, Reg8::L),
            0x66 => self.ld_from_hl(Reg8::H, mmu),
            0x67 => self.ld(Reg8::H, Reg8::A),

            0x68 => self.ld(Reg8::L, Reg8::B),
            0x69 => self.ld(Reg8::L, Reg8::C),
            0x6A => self.ld(Reg8::L, Reg8::D),
            0x6B => self.ld(Reg8::L, Reg8::E),
            0x6C => self.ld(Reg8::L, Reg8::H),
            0x6D => self.ld(Reg8::L, Reg8::L),
            0x6E => self.ld_from_hl(Reg8::L, mmu),
            0x6F => self.ld(Reg8::L, Reg8::A),

            0x70 => self.ld_to_hl(Reg8::B, mmu),
            0x71 => self.ld_to_hl(Reg8::C, mmu),
            0x72 => self.ld_to_hl(Reg8::D, mmu),
            0x73 => self.ld_to_hl(Reg8::E, mmu),
            0x74 => self.ld_to_hl(Reg8::H, mmu),
            0x75 => self.ld_to_hl(Reg8::L, mmu),
            0x76 => self.halt(),
            0x77 => self.ld_to_hl(Reg8::A, mmu),

            0x78 => self.ld(Reg8::A, Reg8::B),
            0x79 => self.ld(Reg8::A, Reg8::C),
            0x7A => self.ld(Reg8::A, Reg8::D),
            0x7B => self.ld(Reg8::A, Reg8::E),
            0x7C => self.ld(Reg8::A, Reg8::H),
            0x7D => self.ld(Reg8::A, Reg8::L),
            0x7E => self.ld_from_hl(Reg8::A, mmu),
            0x7F => self.ld(Reg8::A, Reg8::A),

            0x80 => self.add(Reg8::B),
            0x81 => self.add(Reg8::C),
            0x82 => self.add(Reg8::D),
            0x83 => self.add(Reg8::E),
            0x84 => self.add(Reg8::H),
            0x85 => self.add(Reg8::L),
            0x86 => self.add_hl(mmu),
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
            0x96 => self.sub_hl(mmu),
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
            0xAE => self.xor_hl(mmu),
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
            0xBE => self.cp_hl(mmu),
            0xBF => self.cp(Reg8::A),

            0xC0 => self.ret(JF::NZ, mmu),
            0xC1 => self.pop(Reg16::BC, mmu),
            0xC2 => self.jp(JF::NZ, mmu),
            0xC3 => self.jp(JF::Always, mmu),
            0xC4 => {
                // TODO CALL NZ, a16
                if self.jump_match(JF::NZ) {
                    let pc = self.reg_pc;
                    let addr = self.read_word(mmu, pc);
                    self.reg_sp -= 2;
                    let sp = self.reg_sp;
                    self.write_word(mmu, sp, pc + 2);
                    self.reg_pc = addr;
                    self.last_t = 24;
                } else {
                    self.reg_pc += 2;
                    self.last_t = 12;
                }
            },
            0xC5 => self.push(Reg16::BC, mmu),
            0xC6 => self.addi(false, mmu),
            0xC7 => self.rst(0x00, mmu),

            0xC8 => self.ret(JF::Z, mmu),
            0xC9 => self.ret(JF::Always, mmu),
            0xCA => self.jp(JF::Z, mmu),
            0xCB => {
                // TODO 0xCB instructions
                let pc = self.reg_pc;
                let op = self.read_byte(mmu, pc);
                self.reg_pc = pc.saturating_add(1);
                match op {
                    0x10 => self.rl(Reg8::B),
                    0x11 => self.rl(Reg8::C),
                    0x12 => self.rl(Reg8::D),
                    0x13 => self.rl(Reg8::E),
                    0x14 => self.rl(Reg8::H),
                    0x15 => self.rl(Reg8::L),

                    0x17 => self.rl(Reg8::C),

                    0x18 => self.rr(Reg8::B),
                    0x19 => self.rr(Reg8::C),
                    0x1A => self.rr(Reg8::D),
                    0x1B => self.rr(Reg8::E),
                    0x1C => self.rr(Reg8::H),
                    0x1D => self.rr(Reg8::L),

                    0x1F => self.rr(Reg8::A),

                    0x30 => self.swap(Reg8::B),
                    0x31 => self.swap(Reg8::C),
                    0x32 => self.swap(Reg8::D),
                    0x33 => self.swap(Reg8::E),
                    0x34 => self.swap(Reg8::H),
                    0x35 => self.swap(Reg8::L),

                    0x37 => self.swap(Reg8::A),

                    0x40 => self.bit(0, Reg8::B),
                    0x41 => self.bit(0, Reg8::C),
                    0x42 => self.bit(0, Reg8::D),
                    0x43 => self.bit(0, Reg8::E),
                    0x44 => self.bit(0, Reg8::H),
                    0x45 => self.bit(0, Reg8::L),

                    0x47 => self.bit(0, Reg8::A),

                    0x48 => self.bit(1, Reg8::B),
                    0x49 => self.bit(1, Reg8::C),
                    0x4A => self.bit(1, Reg8::D),
                    0x4B => self.bit(1, Reg8::E),
                    0x4C => self.bit(1, Reg8::H),
                    0x4D => self.bit(1, Reg8::L),

                    0x4F => self.bit(1, Reg8::A),

                    0x6F => self.bit(5, Reg8::A),

                    0x70 => self.bit(6, Reg8::B),

                    0x7C => self.bit(7, Reg8::H),

                    0x80 => self.res(0, Reg8::B),
                    0x81 => self.res(0, Reg8::C),
                    0x82 => self.res(0, Reg8::D),
                    0x83 => self.res(0, Reg8::E),
                    0x84 => self.res(0, Reg8::H),
                    0x85 => self.res(0, Reg8::L),
//                    0x86 => self.res(0, Reg8::B), // (HL)
                    0x87 => self.res(0, Reg8::A),

                    0x88 => self.res(1, Reg8::B),
                    0x89 => self.res(1, Reg8::C),
                    0x8A => self.res(1, Reg8::D),
                    0x8B => self.res(1, Reg8::E),
                    0x8C => self.res(1, Reg8::H),
                    0x8D => self.res(1, Reg8::L),
                    0x8F => self.res(1, Reg8::A),

                    0x90 => self.res(2, Reg8::B),
                    0x91 => self.res(2, Reg8::C),
                    0x92 => self.res(2, Reg8::D),
                    0x93 => self.res(2, Reg8::E),
                    0x94 => self.res(2, Reg8::H),
                    0x95 => self.res(2, Reg8::L),
                    0x97 => self.res(2, Reg8::A),

                    0x98 => self.res(3, Reg8::B),
                    0x99 => self.res(3, Reg8::C),
                    0x9A => self.res(3, Reg8::D),
                    0x9B => self.res(3, Reg8::E),
                    0x9C => self.res(3, Reg8::H),
                    0x9D => self.res(3, Reg8::L),
                    0x9F => self.res(3, Reg8::A),

                    0xAF => self.res(5, Reg8::A),

                    _ => panic!("Unknown CB op: {:#X} at addr: {:#X}", op, pc - 1),
                }
            },
            0xCC => {
                // CALL Z,a16
                if self.flag_reg.zero {
                    let pc = self.reg_pc;
                    let addr = self.read_word(mmu, pc);
                    self.reg_sp -= 2;
                    let sp = self.reg_sp;
                    self.write_word(mmu, sp, pc + 2);
                    self.reg_pc = addr;
                    self.last_t = 24;
                } else {
                    self.reg_pc += 2;
                    self.last_t = 12;
                }
            }
            0xCD => {
                // TODO CALL a16
                let pc = self.reg_pc;
                let addr = self.read_word(mmu, pc);
                self.push(Reg16::PC, mmu);
                self.reg_pc = addr;
                self.last_t = 24;
            }
            0xCE => self.addi(true, mmu),
            0xCF => self.rst(0x08, mmu),

            0xD0 => self.ret(JF::NC, mmu),
            0xD1 => self.pop(Reg16::DE, mmu),
            0xD2 => self.jp(JF::NC, mmu),
            0xD5 => self.push(Reg16::DE, mmu),
            0xD6 => self.subi(mmu),
            0xD7 => self.rst(0x10, mmu),

            0xD8 => self.ret(JF::C, mmu),
            0xD9 => {
                self.ret(JF::Always, mmu);
                self.ime = true;
            }
            0xDA => self.jp(JF::C, mmu),
            0xDF => self.rst(0x18, mmu),

            0xE0 => {
                // TODO LDH (a8),A
                let pc = self.reg_pc;
                let offset = self.read_byte(mmu, pc);
                let addr = 0xFF00 + offset as u16;
                let value = self.read_reg(Reg8::A);
                self.reg_pc += 1;
                self.write_byte(mmu, addr, value);
                self.last_t = 12;
                println!("LDH ({:#X}),A", addr);
            }
            0xE1 => self.pop(Reg16::HL, mmu),
            0xE2 => {
                // TODO LD (C),A
                println!("LD (0xFF00 + C), A");
                let value = self.read_reg(Reg8::A);
                let addr = (self.reg_c as u16) + 0xFF00;
                self.write_byte(mmu, addr, value);
                self.last_t = 8;
            }
            0xE5 => self.push(Reg16::HL, mmu),
            0xE6 => self.andi(mmu),
            0xE7 => self.rst(0x20, mmu),
            0xE9 => {
                // TODO JP (HL)
                println!("JP (HL)");
                let addr = self.read_reg16(Reg16::HL);
                self.reg_pc = addr;
                self.last_t = 4;
            }
            0xEA => {
                // TODO LD (a16),A
                let pc = self.reg_pc;
                let addr = self.read_word(mmu, pc);
                self.reg_pc += 2;
                let value = self.read_reg(Reg8::A);
                self.write_byte(mmu, addr, value);
                self.last_t = 16;
                println!("LD ({:#X}), A", addr);
            }
            0xED => {
                let pc = self.reg_pc;
                let val = self.read_word(mmu, pc);
                println!("{:?}", self);
                panic!("Opcode 0xED is invalid! {:#X}", val)
            }
            0xEF => self.rst(0x28, mmu),
            0xF0 => {
                // TODO LDH A,(a8)
                let pc = self.reg_pc;
                let offset = self.read_byte(mmu, pc);
                let addr = 0xFF00 + offset as u16;
                let value = self.read_byte(mmu, addr);
                self.reg_pc += 1;
                self.reg_a = value;
                self.last_t = 12;
                println!("LDH A,({:#X})", addr);
            }
            0xF1 => self.pop(Reg16::AF, mmu),
            0xF3 => {
                // Disable Interrupts
                self.ime = false;
                self.last_t = 4;
            }
            0xF5 => self.push(Reg16::AF, mmu),
            0xF7 => self.rst(0x30, mmu),
            0xFA => { // LD A, (a16)
                let pc = self.reg_pc;
                let addr = self.read_word(mmu, pc);
                self.reg_pc += 2;
                let value = self.read_byte(mmu, addr);
                self.reg_a = value;
                self.last_t = 12;
            }
            0xFB => {
                // Enable Interrupts
                self.ime = true;
                self.last_t = 4;
            }

            0xFE => { // CP d8
                self.last_t = 8;
                let pc = self.reg_pc;
                let value = self.read_byte(mmu, pc);
                self.reg_pc += 1;
                self.cpi(value);
            }
            0xFF => self.rst(0x38, mmu),

            _ => panic!("Unknown opcode: {:#X} at address {:#X}", opcode, self.reg_pc - 1)
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Flags {
    zero:  bool,
    sub:   bool,
    half:  bool,
    carry: bool,
}

impl Flags {
    fn into(self) -> u8 {
        let mut result = 0;
        if self.zero  { result |= 1 << 7; }
        if self.sub   { result |= 1 << 6; }
        if self.half  { result |= 1 << 5; }
        if self.carry { result |= 1 << 4; }
        result
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Flags {
            zero:  value & (1 << 7) != 0,
            sub:   value & (1 << 6) != 0,
            half:  value & (1 << 5) != 0,
            carry: value & (1 << 4) != 0,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Reg8 {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
//    AtHL,
}

#[derive(Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Copy, Clone)]
pub enum JF { // Jump flags
    Always,
    Z,
    C,
    NZ,
    NC
}

pub enum ID { // Inc/Dec HL
    None,
    Inc,
    Dec
}
