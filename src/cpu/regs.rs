#[derive(Default)]
pub struct Regs {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    flags: Flags
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
        self.flag_carry = (old_a as u16) + (value as u16) > 255;
    }

    pub fn add_HL(&mut self, reg: Reg16) {
        let old = self.read16(Reg16::HL);
        let value = self.read16(reg);
        let result = old.wrapping_add(value);
        self.write16(result, Reg16::HL);
        self.flag_sub = false;
        self.flag_half =
            (old & 0x0F00).wrapping_add(other_reg & 0x0F00) & 0x1000 == 0x1000;
        self.flag_carry = (old as u32) + (other_reg as u32) >= 0x10000;
    }

    pub fn addi(&mut self, value: u8, carry: bool) {
        let old_a = self.read(Reg8::A);
        let result = if carry && self.flags.carry {
            old_a.wrapping_add(value).wrapping_add(1);
            self.flag_carry = (old_a as u16) + (value as u16) + 1 > 255
        } else {
            old_a.wrapping_add(value);
            self.flag_carry = (old_a as u16) + (value as u16) > 255;
        }
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
            self.flag_zero = old == 0xFF;
            self.flag_sub = false;
            self.flag_half = (old & 0x0F + 1) & 0x10 == 0x10;
            self.flag_carry = (old as u16) + 1 > 255;
        }
    }

    pub fn and(&mut self, reg: Reg8) {
        let value = self.read(Reg::A) & self.read(reg);
        self.write(value, Reg::A);
        self.flags.zero = result == 0x00;
        self.flags.sub = false;
        self.flags.half = true;
        self.flags.carry = false;
    }

    pub fn read(&mut self, reg: Reg8) -> u8 {
        match reg {
            Reg8::A => self.a,
            Reg8::B => self.b,
            Reg8::C => self.c,
            Reg8::D => self.d,
            Reg8::E => self.e,
            Reg8::F => self.flags.into(),
            Reg8::H => self.h,
            Reg8::L => self.l,
        }
    }

    pub fn read16(&mut self, reg: Reg16) -> u16 {
        match reg {
            Reg16::BC =>
                (self.read(Reg8::B) as u16) << 8 | self.read(Reg8::C) as u16
            Reg16::DE =>
                (self.read(Reg8::D) as u16) << 8 | self.read(Reg8::E) as u16
            Reg16::HL =>
                (self.read(Reg8::H) as u16) << 8 | self.read(Reg8::L) as u16
            Reg16::SP => self.sp,
            Reg16::PC => self.pc,
            Reg16::AF =>
                (self.read(Reg8::A) as u16) << 8 | self.flags.into() as u16
        }
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
}

struct Flags {
    zero:  bool,
    sub:   bool,
    half:  bool,
    carry: bool,
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        self.zero  = value & (1 << 7) != 0;
        self.sub   = value & (1 << 6) != 0;
        self.half  = value & (1 << 5) != 0;
        self.carry = value & (1 << 4) != 0;
    }
}

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

pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

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
