#[derive(Debug)]
pub struct Regs {
    pub pc: u16,
    pub sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    flags: Flags
}

impl Default for Regs {
    fn default() -> Self {
        Regs {
            pc: 0x0100,
            sp: 0xFFFE,
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            flags: Flags::default(),
        }
    }
}

impl Regs {
    pub fn add(&mut self, reg: Reg8) {
        let old_a = self.read(Reg8::A);
        let value = self.read(reg);
        let result = old_a.wrapping_add(value);
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = false;
        self.flags.half = (old_a & 0x0F + value & 0x0F) & 0x10 == 0x10;
        self.flags.carry = (old_a as u16) + (value as u16) > 255;
    }

    pub fn add_to_HL(&mut self, reg: Reg16) {
        let old = self.read16(Reg16::HL);
        let value = self.read16(reg);
        let result = old.wrapping_add(value);
        self.write16(result, Reg16::HL);
        self.flags.sub = false;
        self.flags.half =
            (old & 0x0F00).wrapping_add(value & 0x0F00) & 0x1000 == 0x1000;
        self.flags.carry = (old as u32) + (value as u32) >= 0x10000;
    }

    pub fn add_HL(&mut self, value: u8) {
        let old = self.a;
        let result = old.wrapping_add(value);
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = false;
        self.flags.half = (old & 0x0F + value & 0x0F) & 0x10 == 0x10;
        self.flags.carry = (old as u16) + (value as u16) > 255;
    }

    pub fn addi(&mut self, value: u8, carry: bool) {
        let old_a = self.read(Reg8::A);
        let result = if carry && self.flags.carry {
            self.flags.carry = (old_a as u16) + (value as u16) + 1 > 255;
            old_a.wrapping_add(value).wrapping_add(1)
        } else {
            self.flags.carry = (old_a as u16) + (value as u16) > 255;
            old_a.wrapping_add(value)
        };
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = false;
        self.flags.half = (old_a & 0x0F + value & 0x0F) & 0x10 == 0x10;
    }

    pub fn adc(&mut self, reg: Reg8) {
        self.add(reg);
        if self.flags.carry {
            let old = self.read(Reg8::A);
            self.write(old.wrapping_add(1), Reg8::A);
            self.flags.zero = old == 0xFF;
            self.flags.sub = false;
            self.flags.half = (old & 0x0F + 1) & 0x10 == 0x10;
            self.flags.carry = (old as u16) + 1 > 255;
        }
    }

    pub fn and(&mut self, reg: Reg8) {
        let value = self.read(Reg8::A) & self.read(reg);
        self.write(value, Reg8::A);
        self.flags.zero = value == 0x00;
        self.flags.sub = false;
        self.flags.half = true;
        self.flags.carry = false;
    }

    pub fn andi(&mut self, imm: u8) {
        let old = self.a;
        let result = old & imm;
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = false;
        self.flags.half = true;
        self.flags.carry = false;
    }

    pub fn bit(&mut self, bit: u8, reg: Reg8) {
        let value = self.read(reg);
        let result = value & (1 << bit) != 0;
        self.flags.zero = !result;
        self.flags.sub = false;
        self.flags.half = true;
    }

    pub fn cp(&mut self, reg: Reg8) {
        let old = self.a;
        let value = self.read(reg);
        let result = old.wrapping_sub(value);
        self.flags.zero = result == 0x00;
        self.flags.sub = true;
        self.flags.half = old & 0x0F < value & 0x0F;
        self.flags.carry = old < value;
    }

    pub fn cp_HL(&mut self, value: u8) {
        let old = self.a;
        let result = old.wrapping_sub(value);
        self.flags.zero = result == 0x00;
        self.flags.sub = true;
        self.flags.half = old & 0x0F < value & 0x0F;
        self.flags.carry = old < value;
    }

    pub fn cpi(&mut self, imm: u8) {
        let old = self.a;
        let result = old.wrapping_sub(imm);
        self.flags.zero = result == 0x00;
        self.flags.sub = true;
        self.flags.half = old & 0x0F < imm;
        self.flags.carry = old < imm;
    }

    pub fn cpl(&mut self) {
        let old = self.a;
        self.write(old ^ 0xFF, Reg8::A);
        self.flags.sub = true;
        self.flags.half = true;
    }

    pub fn dec(&mut self, reg: Reg8) {
        let old = self.read(reg);
        let value = old.wrapping_sub(1);
        self.write(value, reg);
        self.flags.zero = value == 0x00;
        self.flags.sub = true;
        self.flags.half = old & 0x0F < 0x01;
    }

    pub fn dec16(&mut self, reg: Reg16) {
        let old = self.read16(reg);
        let value = old.wrapping_sub(1);
        self.write16(value, reg);
    }

    pub fn inc(&mut self, reg: Reg8) {
        let old = self.read(reg);
        let value = old.wrapping_add(1);
        self.write(value, reg);
        self.flags.zero = value == 0x00;
        self.flags.sub = false;
        self.flags.half = old & 0x0F + 1 == 0x10;
    }

    pub fn inc16(&mut self, reg: Reg16) {
        let old = self.read16(reg);
        let value = old.wrapping_add(1);
        self.write16(value, reg);
    }

    pub fn jump_match(&self, flag: JF) -> bool {
        match flag {
            JF::Always => true,
            JF::Z => self.flags.zero,
            JF::C => self.flags.carry,
            JF::NZ => !self.flags.zero,
            JF::NC => !self.flags.carry,
        }
    }

    pub fn ld(&mut self, rd: Reg8, rs: Reg8) {
        let value = self.read(rs);
        self.write(value, rd);
    }

    pub fn or(&mut self, reg: Reg8) {
        let value = self.read(Reg8::A) & self.read(reg);
        self.write(value, Reg8::A);
        self.flags.zero = value == 0x00;
        self.flags.sub = false;
        self.flags.half = false;
        self.flags.carry = false;
    }

    pub fn read(&mut self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::F => self.flags.clone().into(),
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn read16(&mut self, reg: Reg16) -> u16 {
        match reg {
            Reg16::BC =>
                (self.read(Reg8::B) as u16) << 8 | self.read(Reg8::C) as u16,
            Reg16::DE =>
                (self.read(Reg8::D) as u16) << 8 | self.read(Reg8::E) as u16,
            Reg16::HL =>
                (self.read(Reg8::H) as u16) << 8 | self.read(Reg8::L) as u16,
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
            Reg16::AF =>
                (self.read(Reg8::A) as u16) << 8 | self.flags.clone().into() as u16,
        }
    }

    pub fn res(&mut self, bit: u8, reg: Reg8) {
        let old = self.read(reg);
        let result = old & !(1 << bit);
        self.write(result, reg);
    }

    pub fn rl(&mut self, reg: Reg8) {
        let carrybit = self.flags.carry;
        let value = self.read(reg);
        let result = value << 1 | (if carrybit { 0b1 } else { 0b0 });
        self.write(result, reg);
        self.flags.zero = result == 0;
        self.flags.sub = false;
        self.flags.half = false;
        self.flags.carry = value & (1 << 7) != 0;
    }

    pub fn rla(&mut self) {
        self.rl(Reg8::A);
        self.flags.zero = false;
    }

    pub fn sub(&mut self, reg: Reg8) {
        let old = self.read(Reg8::A);
        let value = self.read(reg);
        let result = old.wrapping_sub(value);
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = true;
        self.flags.half = old & 0x0F < value & 0x0F;
        self.flags.carry = old < value;
    }

    pub fn subi(&mut self, imm: u8) {
        let old = self.read(Reg8::A);
        let result = old.wrapping_sub(imm);
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = true;
        self.flags.half = old & 0x0F < imm & 0x0F;
        self.flags.carry = old < imm;
    }

    pub fn sub_HL(&mut self, value: u8) {
        let old = self.a;
        let result = old.wrapping_sub(value);
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = true;
        self.flags.half = old & 0x0F < value & 0x0F;
        self.flags.carry = old < value;
    }

    pub fn sbc(&mut self, reg: Reg8) {
        self.sub(reg);
        if self.flags.carry {
            let old = self.read(Reg8::A);
            let result = old.wrapping_sub(1);
            self.write(result, Reg8::A);
            self.flags.zero = result == 0x00;
            self.flags.sub = true;
            self.flags.half = old & 0x0F < 1;
            self.flags.carry = old < 1;
        }
    }


    pub fn swap(&mut self, reg: Reg8) {
        let old = self.read(reg);
        let lo = old & 0x0F;
        let hi = old & 0xF0;
        let result = lo << 4 | hi >> 4;
        self.write(result, reg);
        self.flags.zero = result == 0x00;
        self.flags.sub = false;
        self.flags.half = false;
        self.flags.carry = false;
    }

    pub fn write(&mut self, value: u8, reg: Reg8) {
        match reg {
            Reg8::A => self.a = value,
            Reg8::B => self.b = value,
            Reg8::C => self.c = value,
            Reg8::D => self.d = value,
            Reg8::E => self.e = value,
            Reg8::F => self.flags = value.into(),
            Reg8::H => self.h = value,
            Reg8::L => self.l = value,
        }
    }

    pub fn write16(&mut self, value: u16, reg: Reg16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        match reg {
            Reg16::BC => {
                self.write(hi, Reg8::B);
                self.write(lo, Reg8::C);
            }
            Reg16::DE => {
                self.write(hi, Reg8::D);
                self.write(lo, Reg8::E);
            }
            Reg16::HL => {
                self.write(hi, Reg8::H);
                self.write(lo, Reg8::L);
            }
            Reg16::SP => self.sp = value,
            Reg16::PC => self.pc = value,
            Reg16::AF => {
                self.write(hi, Reg8::A);
                self.write(lo, Reg8::F);
            }
        }
    }

    pub fn xor(&mut self, reg: Reg8) {
        let old = self.read(Reg8::A);
        let value = self.read(reg);
        let result = old ^ value;
        self.write(value, Reg8::A);
        self.flags.zero = result == 0;
        self.flags.sub = false;
        self.flags.half = false;
        self.flags.carry = false;
    }

    pub fn xor_HL(&mut self, value: u8) {
        let old = self.a;
        let result = old ^ value;
        self.write(result, Reg8::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = false;
        self.flags.half = false;
        self.flags.carry = false;
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct Flags {
    zero:  bool,
    sub:   bool,
    half:  bool,
    carry: bool,
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
