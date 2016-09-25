use dmg::Interconnect;
use super::opcode::{Opcode, Reg8, Reg16, JF, ID};

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
    ime_next: bool,
    // halted, waiting for interrupt
    pub halted: bool,
    // stopped, waiting for button press
    pub stopped: bool,
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
            ime_next: false,
            halted: false,
            stopped: false,
            // clock time of last instruction
            last_t: 0,
            // clock time total
            clock_t: 0,
        }
    }

    pub fn current_pc(&self) -> u16 {
        self.reg_pc
    }

    pub fn step(&mut self, interconnect: &mut Interconnect) -> usize {
        if self.halted || self.stopped {
            4 // wait 4 cycles at a time for interrupt/button press
        } else {
            if self.ime_next {
                self.ime_next = false;
                self.ime = true;
            }
            let pc = self.reg_pc;
            let opcode = interconnect.read_byte(pc);
            self.reg_pc = pc.saturating_add(1);

            println!("PC: {:#x}", pc);

            self.execute_instr(opcode, interconnect);

            let cycles = self.last_t;
            self.clock_t += cycles;
            cycles
        }
    }

    pub fn interrupt(&mut self, addr: u16, interconnect: &mut Interconnect) -> usize {
        self.ime = false;
        let cycles = 16;
        self.last_t = cycles;
        self.clock_t += cycles;
        self.push(Reg16::PC, interconnect);
        self.reg_pc = addr;
        cycles
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

    fn adc_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = interconnect.read_byte(addr);
        let old = self.reg_a;
        let carry = if self.flag_reg.carry { 1 } else { 0 };
        let result = old.wrapping_add(value).wrapping_add(carry);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = (old & 0xF) + (value.wrapping_add(carry) & 0xF) & 0x10 == 0x10;
        self.flag_reg.carry = (old as u16) + (value as u16) + (carry as u16) > 255;
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

    fn add_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(interconnect, addr);
        let old = self.reg_a;
        let result = old.wrapping_add(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = (old & 0x0F + value & 0x0F) & 0x10 == 0x10;
        self.flag_reg.carry = (old as u16) + (value as u16) > 255;
    }

    fn addi(&mut self, carry: bool, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.reg_pc;
        let value = self.read_byte(interconnect, addr);
        self.reg_pc += 1;
        let old = self.reg_a;
        let result = if carry && self.flag_reg.carry {
            self.flag_reg.carry = (old as u16) + (value as u16).wrapping_add(1) > 255;
            self.flag_reg.half =
                (old & 0x0F + value.wrapping_add(1) & 0x0F) & 0x10 == 0x10;
            old.wrapping_add(value).wrapping_add(1)
        } else {
            self.flag_reg.carry = (old as u16) + (value as u16) > 255;
            self.flag_reg.half = (old & 0x0F + value & 0x0F) & 0x10 == 0x10;
            old.wrapping_add(value)
        };
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
    }

    fn andi(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = self.read_byte(interconnect, pc);
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

    fn and_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = interconnect.read_byte(addr);
        let result = self.reg_a & value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
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

    fn bit_hl(&mut self, bit: u8, interconnect: &mut Interconnect) {
        self.last_t = 12;
        let addr = self.read_reg16(Reg16::HL);
        let value = interconnect.read_byte(addr);
        let result = value & (1 << bit) != 0;
        self.flag_reg.zero = !result;
        self.flag_reg.sub = false;
        self.flag_reg.half = true;
    }

    fn call (&mut self, flag: JF, interconnect: &mut Interconnect) {
        let pc = self.reg_pc;
        let addr = self.read_word(interconnect, pc);
        self.reg_pc += 2;
        let jump = self.jump_match(flag);
        if jump {
            self.push(Reg16::PC, interconnect);
            self.reg_pc = addr;
            self.last_t = 24;
        } else {
            self.last_t = 12;
        }
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

    fn cp_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(interconnect, addr);
        let old = self.reg_a;
        let result = old.wrapping_sub(value);
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value & 0x0F;
        self.flag_reg.carry = old < value;
    }

    fn daa(&mut self) {
        // decimal adjust accumulator
        // TODO check this over
        self.last_t = 4;
        let mut value = 0u8;
        if !self.flag_reg.sub {
            if self.flag_reg.half || (self.reg_a & 0xF) > 9 {
                value += 0x06;
            }
            if self.flag_reg.carry || (self.reg_a > 0x9F) {
                value += 0x60;
            }
        } else {
            if self.flag_reg.half {
                value = value.wrapping_sub(0x06);
            }
            if self.flag_reg.carry {
                value = value.wrapping_sub(0x60);
            }
        };
        let old = self.reg_a;
        let result = old.wrapping_add(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.half = false;
        self.flag_reg.carry = ((old as u16) + (value as u16)) & 0x100 == 0x100;
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

    fn jr(&mut self, flag: JF, interconnect: &mut Interconnect) {
        let pc = self.reg_pc;
        let rel_addr = self.read_byte(interconnect, pc) as i8;
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

    fn jp(&mut self, flag: JF, interconnect: &mut Interconnect) {
        let pc = self.reg_pc;
        let addr = self.read_word(interconnect, pc);
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

    fn ldi(&mut self, reg: Reg8, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = self.read_byte(interconnect, pc);
        self.reg_pc += 1;
        self.write_reg(reg, value);
    }

    fn ldi16(&mut self, reg: Reg16, interconnect: &mut Interconnect) {
        self.last_t = 12;
        let pc = self.reg_pc;
        let value = self.read_word(interconnect, pc);
        self.reg_pc += 2;
        self.write_reg16(reg, value);
    }

    fn ld_from_hl(&mut self, reg: Reg8, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(interconnect, addr);
        self.write_reg(reg, value);
    }

    fn ld_to_hl(&mut self, reg: Reg8, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_reg(reg);
        self.write_byte(interconnect, addr, value);
    }

    fn ld_from_mem(&mut self, reg: Reg16, id: ID, interconnect: &mut Interconnect) {
        let addr = self.read_reg16(reg);
        let next_addr = match id {
            ID::None => addr,
            ID::Inc => addr + 1,
            ID::Dec => addr - 1,
        };
        self.write_reg16(reg, next_addr);
        let value = self.read_byte(interconnect, addr);
        self.reg_a = value;
        self.last_t = 8;
    }

    fn ld_to_mem(&mut self, reg: Reg16, id: ID, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let value = self.reg_a;
        let addr = self.read_reg16(reg);
        let next_addr = match id {
            ID::None => addr,
            ID::Inc => addr + 1,
            ID::Dec => addr - 1,
        };
        self.write_reg16(reg, next_addr);
        self.write_byte(interconnect, addr, value);
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

    fn ori(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = interconnect.read_byte(pc);
        self.reg_pc += 1;
        let result = self.reg_a | value;
        self.reg_a = result;
        self.flag_reg.zero = value == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn or_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = interconnect.read_byte(addr);
        let result = self.reg_a | value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn pop(&mut self, reg: Reg16, interconnect: &mut Interconnect) {
        let sp = self.reg_sp;
        let value = self.read_word(interconnect, sp);
        self.write_reg16(reg, value);
        self.reg_sp += 2;
        self.last_t = 12;
    }

    fn push(&mut self, reg: Reg16, interconnect: &mut Interconnect) {
        let sp = self.reg_sp - 2;
        let value = self.read_reg16(reg);
        self.write_word(interconnect, sp, value);
        self.reg_sp = sp;
        self.last_t = 16;
    }

    fn res(&mut self, bit: u8, reg: Reg8) {
        self.last_t = 8;
        let old = self.read_reg(reg);
        let result = old & !(1 << bit);
        self.write_reg(reg, result);
    }

    fn res_hl(&mut self, bit: u8, interconnect: &mut Interconnect) {
        self.last_t = 16;
        let addr = self.read_reg16(Reg16::HL);
        let value = interconnect.read_byte(addr);
        let result = value & !(1 << bit);
        interconnect.write_byte(addr, result);
    }

    fn ret(&mut self, flag: JF, interconnect: &mut Interconnect) {
        let jump = self.jump_match(flag);
        if jump {
            let sp = self.reg_sp;
            let addr = self.read_word(interconnect, sp);
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

    fn rl_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 16;
        let carrybit = self.flag_reg.carry;
        let addr = self.read_reg16(Reg16::HL);
        let old = interconnect.read_byte(addr);
        let result = old << 1 | if carrybit { 1 } else { 0 };
        interconnect.write_byte(addr, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = old & (1 << 7) != 0;
    }

    fn rla(&mut self) {
        self.rl(Reg8::A);
        self.last_t = 4;
        self.flag_reg.zero = false;
    }

    fn rlc(&mut self, reg: Reg8) {
        self.last_t = 8;
        let old = self.read_reg(reg);
        self.flag_reg.carry = old >> 7 & 1 != 0;
        let result = old.rotate_left(1);
        self.write_reg(reg, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
    }

    fn rlc_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 16;
        let addr = self.read_reg16(Reg16::HL);
        let old = interconnect.read_byte(addr);
        self.flag_reg.carry = old >> 7 & 1 != 0;
        let result = old.rotate_left(1);
        interconnect.write_byte(addr, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
    }

    fn rlca(&mut self) {
        self.rlc(Reg8::A);
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

    fn rr_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 16;
        let carrybit = self.flag_reg.carry;
        let addr = self.read_reg16(Reg16::HL);
        let old = interconnect.read_byte(addr);
        let result = old >> 1 | if carrybit { 1 << 7 } else { 0 };
        interconnect.write_byte(addr, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = old & 1 != 0;
    }

    fn rra(&mut self) {
        self.rr(Reg8::A);
        self.last_t = 4;
        self.flag_reg.zero = false;
    }

    fn rrc(&mut self, reg: Reg8) {
        self.last_t = 8;
        let old = self.read_reg(reg);
        self.flag_reg.carry = old & 1 != 0;
        let result = old.rotate_right(1);
        self.write_reg(reg, result);
        self.flag_reg.zero = result == 0;
    }

    fn rrc_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 16;
        let addr = self.read_reg16(Reg16::HL);
        let old = interconnect.read_byte(addr);
        self.flag_reg.carry = old & 1 != 0;
        let result = old.rotate_right(1);
        interconnect.write_byte(addr, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
    }

    fn rrca(&mut self) {
        self.rrc(Reg8::A);
        self.last_t = 4;
        self.flag_reg.zero = false;
    }

    fn rst(&mut self, addr: u16, interconnect: &mut Interconnect) {
        self.last_t = 16;
        let sp = self.reg_sp - 2;
        let pc = self.reg_pc;
        self.write_word(interconnect, sp, pc);
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

    fn sbc_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let old = self.reg_a;
        let value = self.read_byte(interconnect, addr);
        let carry = if self.flag_reg.carry { 1 } else { 0 };
        let result = old.wrapping_sub(value).wrapping_add(carry);
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value.wrapping_add(carry) & 0x0F;
        self.flag_reg.carry = old < value.wrapping_add(carry);
    }

    fn set(&mut self, bit: u8, reg: Reg8) {
        self.last_t = 8;
        let old = self.read_reg(reg);
        let result = old | (1 << bit);
        self.write_reg(reg, result);
    }

    fn srl(&mut self, reg: Reg8) {
        self.last_t = 8;
        let old = self.read_reg(reg);
        let result = old >> 1;
        self.write_reg(reg, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = old & 0b1 != 0;
    }

    fn srl_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 16;
        let addr = self.read_reg16(Reg16::HL);
        let old = interconnect.read_byte(addr);
        let result = old >> 1;
        interconnect.write_byte(addr, result);
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = old & 0b1 != 0;
    }

    fn stop(&mut self) {
        self.stopped = true;
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

    fn subi(&mut self, carry: bool, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let imm = self.read_byte(interconnect, pc);
        self.reg_pc += 1;
        let old = self.reg_a;
        let result = if carry && self.flag_reg.carry {
            self.flag_reg.carry = old < imm.wrapping_add(1);
            self.flag_reg.half = old & 0x0F < imm.wrapping_add(1) & 0x0F;
            old.wrapping_sub(imm).wrapping_sub(1)
        } else {
            self.flag_reg.carry = old < imm;
            self.flag_reg.half = old & 0x0F < imm & 0x0F;
            old.wrapping_sub(imm)
        };
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
    }

    fn sub_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let old = self.reg_a;
        let value = self.read_byte(interconnect, addr);
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

    fn xori(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let pc = self.reg_pc;
        let value = interconnect.read_byte(pc);
        self.reg_pc += 1;
        let result = self.reg_a ^ value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn xor_hl(&mut self, interconnect: &mut Interconnect) {
        self.last_t = 8;
        let addr = self.read_reg16(Reg16::HL);
        let value = self.read_byte(interconnect, addr);
        let result = self.reg_a ^ value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn read_byte(&mut self, interconnect: &mut Interconnect, addr: u16) -> u8 {
        interconnect.read_byte(addr)
    }

    fn read_word(&mut self, interconnect: &mut Interconnect, addr: u16) -> u16 {
        interconnect.read_word(addr)
    }

    fn write_byte(&mut self, interconnect: &mut Interconnect, addr: u16, value: u8) {
        interconnect.write_byte(addr, value);
    }

    fn write_word(&mut self, interconnect: &mut Interconnect, addr: u16, value: u16) {
        interconnect.write_word(addr, value);
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

    fn execute_instr(&mut self, opcode: u8, interconnect: &mut Interconnect) {
        match opcode {
            0x00 => {
                // NOP
                self.last_t = 4;
                println!("NOP");
            },
            0x01 => self.ldi16(Reg16::BC, interconnect),
            0x02 => self.ld_to_mem(Reg16::BC, ID::None, interconnect),
            0x03 => self.inc16(Reg16::BC),
            0x04 => self.inc8(Reg8::B),
            0x05 => self.dec8(Reg8::B),
            0x06 => self.ldi(Reg8::B, interconnect),
            0x07 => self.rlca(),

            0x08 => {
                // LD (a16), SP
                self.last_t = 20;
                let pc = self.reg_pc;
                let addr = interconnect.read_word(pc);
                self.reg_pc += 2;
                let sp = self.reg_sp;
                interconnect.write_word(addr, sp);
            },
            0x09 => self.add_to_hl(Reg16::BC),
            0x10 => self.stop(),
            0x0A => self.ld_from_mem(Reg16::BC, ID::None, interconnect),
            0x0B => self.dec16(Reg16::BC),
            0x0C => self.inc8(Reg8::C),
            0x0D => self.dec8(Reg8::C),
            0x0E => self.ldi(Reg8::C, interconnect),
            0x0F => self.rrca(),

            0x11 => self.ldi16(Reg16::DE, interconnect),
            0x12 => self.ld_to_mem(Reg16::DE, ID::None, interconnect),
            0x13 => self.inc16(Reg16::DE),
            0x14 => self.inc8(Reg8::D),
            0x15 => self.dec8(Reg8::D),
            0x16 => self.ldi(Reg8::D, interconnect),
            0x17 => self.rla(),

            0x18 => self.jr(JF::Always, interconnect),
            0x19 => self.add_to_hl(Reg16::DE),
            0x1A => self.ld_from_mem(Reg16::DE, ID::None, interconnect),
            0x1B => self.dec16(Reg16::DE),
            0x1C => self.inc8(Reg8::E),
            0x1D => self.dec8(Reg8::E),
            0x1E => self.ldi(Reg8::E, interconnect),
            0x1F => self.rra(),

            0x20 => self.jr(JF::NZ, interconnect),
            0x21 => self.ldi16(Reg16::HL, interconnect),
            0x22 => self.ld_to_mem(Reg16::HL, ID::Inc, interconnect),
            0x23 => self.inc16(Reg16::HL),
            0x24 => self.inc8(Reg8::H),
            0x25 => self.dec8(Reg8::H),
            0x26 => self.ldi(Reg8::H, interconnect),
            0x27 => self.daa(),

            0x28 => self.jr(JF::Z, interconnect),
            0x29 => self.add_to_hl(Reg16::HL),
            0x2A => self.ld_from_mem(Reg16::HL, ID::Inc, interconnect),
            0x2B => self.dec16(Reg16::HL),
            0x2C => self.inc8(Reg8::L),
            0x2D => self.dec8(Reg8::L),
            0x2E => self.ldi(Reg8::L, interconnect),
            0x2F => { // CPL
                self.last_t = 4;
                self.reg_a ^= 0xFF;
                self.flag_reg.sub = true;
                self.flag_reg.half = true;
            }

            0x30 => self.jr(JF::NC, interconnect),
            0x31 => self.ldi16(Reg16::SP, interconnect),
            0x32 => self.ld_to_mem(Reg16::HL, ID::Dec, interconnect),
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
                let value = self.read_byte(interconnect, pc);
                self.reg_pc += 1;
                let addr = self.read_reg16(Reg16::HL);
                self.write_byte(interconnect, addr, value);
            }
            0x37 => {
                // SCF
                self.flag_reg.carry = true;
                self.flag_reg.sub = false;
                self.flag_reg.half = false;
                self.last_t = 4;
            }

            0x38 => self.jr(JF::C, interconnect),
            0x39 => self.add_to_hl(Reg16::SP),
            0x3A => self.ld_from_mem(Reg16::HL, ID::Dec, interconnect),
            0x3B => self.dec16(Reg16::SP),
            0x3C => self.inc8(Reg8::A),
            0x3D => self.dec8(Reg8::A),
            0x3E => self.ldi(Reg8::A, interconnect),
            0x3F => {
                // CCF
                self.flag_reg.carry = false;
                self.flag_reg.half = false;
                self.flag_reg.sub = false;
                self.last_t = 4;
            }

            0x40 => self.ld(Reg8::B, Reg8::B),
            0x41 => self.ld(Reg8::B, Reg8::C),
            0x42 => self.ld(Reg8::B, Reg8::D),
            0x43 => self.ld(Reg8::B, Reg8::E),
            0x44 => self.ld(Reg8::B, Reg8::H),
            0x45 => self.ld(Reg8::B, Reg8::L),
            0x46 => self.ld_from_hl(Reg8::B, interconnect),
            0x47 => self.ld(Reg8::B, Reg8::A),

            0x48 => self.ld(Reg8::C, Reg8::B),
            0x49 => self.ld(Reg8::C, Reg8::C),
            0x4A => self.ld(Reg8::C, Reg8::D),
            0x4B => self.ld(Reg8::C, Reg8::E),
            0x4C => self.ld(Reg8::C, Reg8::H),
            0x4D => self.ld(Reg8::C, Reg8::L),
            0x4E => self.ld_from_hl(Reg8::C, interconnect),
            0x4F => self.ld(Reg8::C, Reg8::A),

            0x50 => self.ld(Reg8::D, Reg8::B),
            0x51 => self.ld(Reg8::D, Reg8::C),
            0x52 => self.ld(Reg8::D, Reg8::D),
            0x53 => self.ld(Reg8::D, Reg8::E),
            0x54 => self.ld(Reg8::D, Reg8::H),
            0x55 => self.ld(Reg8::D, Reg8::L),
            0x56 => self.ld_from_hl(Reg8::D, interconnect),
            0x57 => self.ld(Reg8::D, Reg8::A),

            0x58 => self.ld(Reg8::E, Reg8::B),
            0x59 => self.ld(Reg8::E, Reg8::C),
            0x5A => self.ld(Reg8::E, Reg8::D),
            0x5B => self.ld(Reg8::E, Reg8::E),
            0x5C => self.ld(Reg8::E, Reg8::H),
            0x5D => self.ld(Reg8::E, Reg8::L),
            0x5E => self.ld_from_hl(Reg8::E, interconnect),
            0x5F => self.ld(Reg8::E, Reg8::A),

            0x60 => self.ld(Reg8::H, Reg8::B),
            0x61 => self.ld(Reg8::H, Reg8::C),
            0x62 => self.ld(Reg8::H, Reg8::D),
            0x63 => self.ld(Reg8::H, Reg8::E),
            0x64 => self.ld(Reg8::H, Reg8::H),
            0x65 => self.ld(Reg8::H, Reg8::L),
            0x66 => self.ld_from_hl(Reg8::H, interconnect),
            0x67 => self.ld(Reg8::H, Reg8::A),

            0x68 => self.ld(Reg8::L, Reg8::B),
            0x69 => self.ld(Reg8::L, Reg8::C),
            0x6A => self.ld(Reg8::L, Reg8::D),
            0x6B => self.ld(Reg8::L, Reg8::E),
            0x6C => self.ld(Reg8::L, Reg8::H),
            0x6D => self.ld(Reg8::L, Reg8::L),
            0x6E => self.ld_from_hl(Reg8::L, interconnect),
            0x6F => self.ld(Reg8::L, Reg8::A),

            0x70 => self.ld_to_hl(Reg8::B, interconnect),
            0x71 => self.ld_to_hl(Reg8::C, interconnect),
            0x72 => self.ld_to_hl(Reg8::D, interconnect),
            0x73 => self.ld_to_hl(Reg8::E, interconnect),
            0x74 => self.ld_to_hl(Reg8::H, interconnect),
            0x75 => self.ld_to_hl(Reg8::L, interconnect),
            0x76 => self.halt(),
            0x77 => self.ld_to_hl(Reg8::A, interconnect),

            0x78 => self.ld(Reg8::A, Reg8::B),
            0x79 => self.ld(Reg8::A, Reg8::C),
            0x7A => self.ld(Reg8::A, Reg8::D),
            0x7B => self.ld(Reg8::A, Reg8::E),
            0x7C => self.ld(Reg8::A, Reg8::H),
            0x7D => self.ld(Reg8::A, Reg8::L),
            0x7E => self.ld_from_hl(Reg8::A, interconnect),
            0x7F => self.ld(Reg8::A, Reg8::A),

            0x80 => self.add(Reg8::B),
            0x81 => self.add(Reg8::C),
            0x82 => self.add(Reg8::D),
            0x83 => self.add(Reg8::E),
            0x84 => self.add(Reg8::H),
            0x85 => self.add(Reg8::L),
            0x86 => self.add_hl(interconnect),
            0x87 => self.add(Reg8::A),

            0x88 => self.adc(Reg8::B),
            0x89 => self.adc(Reg8::C),
            0x8A => self.adc(Reg8::D),
            0x8B => self.adc(Reg8::E),
            0x8C => self.adc(Reg8::H),
            0x8D => self.adc(Reg8::L),
            0x8E => self.adc_hl(interconnect),
            0x8F => self.adc(Reg8::A),

            0x90 => self.sub(Reg8::B),
            0x91 => self.sub(Reg8::C),
            0x92 => self.sub(Reg8::D),
            0x93 => self.sub(Reg8::E),
            0x94 => self.sub(Reg8::H),
            0x95 => self.sub(Reg8::L),
            0x96 => self.sub_hl(interconnect),
            0x97 => self.sub(Reg8::A),

            0x98 => self.sbc(Reg8::B),
            0x99 => self.sbc(Reg8::C),
            0x9A => self.sbc(Reg8::D),
            0x9B => self.sbc(Reg8::E),
            0x9C => self.sbc(Reg8::H),
            0x9D => self.sbc(Reg8::L),
            0x9E => self.sbc_hl(interconnect),
            0x9F => self.sbc(Reg8::A),

            0xA0 => self.and(Reg8::B),
            0xA1 => self.and(Reg8::C),
            0xA2 => self.and(Reg8::D),
            0xA3 => self.and(Reg8::E),
            0xA4 => self.and(Reg8::H),
            0xA5 => self.and(Reg8::L),
            0xA6 => self.and_hl(interconnect),
            0xA7 => self.and(Reg8::A),

            0xA8 => self.xor(Reg8::B),
            0xA9 => self.xor(Reg8::C),
            0xAA => self.xor(Reg8::D),
            0xAB => self.xor(Reg8::E),
            0xAC => self.xor(Reg8::H),
            0xAD => self.xor(Reg8::L),
            0xAE => self.xor_hl(interconnect),
            0xAF => self.xor(Reg8::A),

            0xB0 => self.or(Reg8::B),
            0xB1 => self.or(Reg8::C),
            0xB2 => self.or(Reg8::D),
            0xB3 => self.or(Reg8::E),
            0xB4 => self.or(Reg8::H),
            0xB5 => self.or(Reg8::L),
            0xB6 => self.or_hl(interconnect),
            0xB7 => self.or(Reg8::A),

            0xB8 => self.cp(Reg8::B),
            0xB9 => self.cp(Reg8::C),
            0xBA => self.cp(Reg8::D),
            0xBB => self.cp(Reg8::E),
            0xBC => self.cp(Reg8::H),
            0xBD => self.cp(Reg8::L),
            0xBE => self.cp_hl(interconnect),
            0xBF => self.cp(Reg8::A),

            0xC0 => self.ret(JF::NZ, interconnect),
            0xC1 => self.pop(Reg16::BC, interconnect),
            0xC2 => self.jp(JF::NZ, interconnect),
            0xC3 => self.jp(JF::Always, interconnect),
            0xC4 => self.call(JF::NZ, interconnect),
            0xC5 => self.push(Reg16::BC, interconnect),
            0xC6 => self.addi(false, interconnect),
            0xC7 => self.rst(0x00, interconnect),

            0xC8 => self.ret(JF::Z, interconnect),
            0xC9 => self.ret(JF::Always, interconnect),
            0xCA => self.jp(JF::Z, interconnect),
            0xCB => self.cb(interconnect),
            0xCC => self.call(JF::Z, interconnect),
            0xCD => self.call(JF::Always, interconnect),
            0xCE => self.addi(true, interconnect),
            0xCF => self.rst(0x08, interconnect),

            0xD0 => self.ret(JF::NC, interconnect),
            0xD1 => self.pop(Reg16::DE, interconnect),
            0xD2 => self.jp(JF::NC, interconnect),
            0xD3 => panic!("No opcode D3!"),
            0xD4 => self.call(JF::NC, interconnect),
            0xD5 => self.push(Reg16::DE, interconnect),
            0xD6 => self.subi(false, interconnect),
            0xD7 => self.rst(0x10, interconnect),

            0xD8 => self.ret(JF::C, interconnect),
            0xD9 => {
                self.ret(JF::Always, interconnect);
                self.ime = true;
            }
            0xDA => self.jp(JF::C, interconnect),
            0xDB => panic!("No opcode DB!"),
            0xDC => self.call(JF::C, interconnect),
            0xDD => panic!("No opcode DD!"),
            0xDE => self.subi(true, interconnect),
            0xDF => self.rst(0x18, interconnect),

            0xE0 => {
                // TODO LDH (a8),A
                let pc = self.reg_pc;
                let offset = self.read_byte(interconnect, pc);
                let addr = 0xFF00 + offset as u16;
                let value = self.reg_a;
                self.reg_pc += 1;
                self.write_byte(interconnect, addr, value);
                self.last_t = 12;
            }
            0xE1 => self.pop(Reg16::HL, interconnect),
            0xE2 => {
                // TODO LD (C),A
                let value = self.read_reg(Reg8::A);
                let addr = (self.reg_c as u16) + 0xFF00;
                self.write_byte(interconnect, addr, value);
                self.last_t = 8;
            }
            0xE3 => panic!("No opcode E3!"),
            0xE4 => panic!("No opcode E4!"),
            0xE5 => self.push(Reg16::HL, interconnect),
            0xE6 => self.andi(interconnect),
            0xE7 => self.rst(0x20, interconnect),

            0xE8 => {
                // add SP, r8
                self.last_t = 16;
                let pc = self.reg_pc;
                let imm = interconnect.read_byte(pc) as i8 as u16;
                self.reg_pc += 1;
                let sp = self.reg_sp;
                self.reg_sp = sp.wrapping_add(imm);
                self.flag_reg.zero = false;
                self.flag_reg.sub = false;
                self.flag_reg.half = (sp & 0xF) + (imm & 0xF) & 0x10 == 0x10;
                self.flag_reg.carry = (sp & 0xFF) + (imm & 0xFF) & 0x100 == 0x100;
            },
            0xE9 => {
                // TODO JP (HL)
                let addr = self.read_reg16(Reg16::HL);
                self.reg_pc = addr;
                self.last_t = 4;
            }
            0xEA => {
                // TODO LD (a16),A
                let pc = self.reg_pc;
                let addr = self.read_word(interconnect, pc);
                self.reg_pc += 2;
                let value = self.read_reg(Reg8::A);
                self.write_byte(interconnect, addr, value);
                self.last_t = 16;
            }
            0xEB => panic!("No opcode EB!"),
            0xEC => panic!("No opcode EC!"),
            0xED => panic!("No opcode ED!"),
            0xEE => self.xori(interconnect),
            0xEF => self.rst(0x28, interconnect),

            0xF0 => {
                // TODO LDH A,(a8)
                let pc = self.reg_pc;
                let offset = self.read_byte(interconnect, pc);
                let addr = 0xFF00 + offset as u16;
                let value = self.read_byte(interconnect, addr);
                self.reg_pc += 1;
                self.reg_a = value;
                self.last_t = 12;
            }
            0xF1 => self.pop(Reg16::AF, interconnect),
            0xF2 => {
                // TODO LD A, (C)
                let addr = (self.reg_c as u16) + 0xFF00;
                let value = interconnect.read_byte(addr);
                self.reg_a = value;
                self.last_t = 8;
            }
            0xF3 => {
                // Disable Interrupts
                self.ime = false;
                self.last_t = 4;
            }
            0xF4 => panic!("No opcode F4!"),
            0xF5 => self.push(Reg16::AF, interconnect),
            0xF6 => self.ori(interconnect),
            0xF7 => self.rst(0x30, interconnect),

            0xF8 => {
                // LD HL, SP+r8
                self.last_t = 12;
                let pc = self.reg_pc;
                let offset = interconnect.read_byte(pc) as i8 as u16;
                self.reg_pc += 1;
                let sp = self.reg_sp;
                let addr = self.reg_sp.wrapping_add(offset);
                self.write_reg16(Reg16::HL, addr);
                self.flag_reg.zero = false;
                self.flag_reg.sub = false;
                self.flag_reg.half = (sp & 0xF) + (offset & 0xF) & 0x10 == 0x10;
                self.flag_reg.carry = (sp & 0xFF) + (offset & 0xFF) & 0x100 == 0x100;
            },
            0xF9 => {
                // LD SP, HL
                self.last_t = 8;
                let value = self.read_reg16(Reg16::HL);
                self.reg_sp = value;
            },
            0xFA => { // LD A, (a16)
                let pc = self.reg_pc;
                let addr = self.read_word(interconnect, pc);
                self.reg_pc += 2;
                let value = self.read_byte(interconnect, addr);
                self.reg_a = value;
                self.last_t = 12;
            }
            0xFB => {
                // Enable Interrupts
                println!("Interrupts Enabled!");
                self.ime_next = true;
                self.last_t = 4;
            }
            0xFC => panic!("No opcode FC!"),
            0xFD => panic!("No opcode FD!"),
            0xFE => { // CP d8
                self.last_t = 8;
                let pc = self.reg_pc;
                let value = self.read_byte(interconnect, pc);
                self.reg_pc += 1;
                self.cpi(value);
            }
            0xFF => self.rst(0x38, interconnect),

            _ => panic!("Unknown opcode: {:#X} at address {:#X}", opcode, self.reg_pc - 1)
        }
    }

    fn cb(&mut self, interconnect: &mut Interconnect) {
        // TODO 0xCB instructions
        let pc = self.reg_pc;
        let op = self.read_byte(interconnect, pc);
        self.reg_pc = pc.saturating_add(1);
        match op {
            0x00 => self.rlc(Reg8::B),
            0x01 => self.rlc(Reg8::C),
            0x02 => self.rlc(Reg8::D),
            0x03 => self.rlc(Reg8::E),
            0x04 => self.rlc(Reg8::H),
            0x05 => self.rlc(Reg8::L),
            0x06 => self.rlc_hl(interconnect),
            0x07 => self.rlc(Reg8::A),

            0x08 => self.rrc(Reg8::B),
            0x09 => self.rrc(Reg8::C),
            0x0A => self.rrc(Reg8::D),
            0x0B => self.rrc(Reg8::E),
            0x0C => self.rrc(Reg8::H),
            0x0D => self.rrc(Reg8::L),
            0x0E => self.rrc_hl(interconnect),
            0x0F => self.rrc(Reg8::A),

            0x10 => self.rl(Reg8::B),
            0x11 => self.rl(Reg8::C),
            0x12 => self.rl(Reg8::D),
            0x13 => self.rl(Reg8::E),
            0x14 => self.rl(Reg8::H),
            0x15 => self.rl(Reg8::L),
            0x16 => self.rl_hl(interconnect),
            0x17 => self.rl(Reg8::C),

            0x18 => self.rr(Reg8::B),
            0x19 => self.rr(Reg8::C),
            0x1A => self.rr(Reg8::D),
            0x1B => self.rr(Reg8::E),
            0x1C => self.rr(Reg8::H),
            0x1D => self.rr(Reg8::L),
            0x1E => self.rr_hl(interconnect),
            0x1F => self.rr(Reg8::A),

            0x30 => self.swap(Reg8::B),
            0x31 => self.swap(Reg8::C),
            0x32 => self.swap(Reg8::D),
            0x33 => self.swap(Reg8::E),
            0x34 => self.swap(Reg8::H),
            0x35 => self.swap(Reg8::L),

            0x37 => self.swap(Reg8::A),

            0x38 => self.srl(Reg8::B),
            0x39 => self.srl(Reg8::C),
            0x3A => self.srl(Reg8::D),
            0x3B => self.srl(Reg8::E),
            0x3C => self.srl(Reg8::H),
            0x3D => self.srl(Reg8::L),
            0x3E => self.srl_hl(interconnect),
            0x3F => self.srl(Reg8::A),

            0x40 => self.bit(0, Reg8::B),
            0x41 => self.bit(0, Reg8::C),
            0x42 => self.bit(0, Reg8::D),
            0x43 => self.bit(0, Reg8::E),
            0x44 => self.bit(0, Reg8::H),
            0x45 => self.bit(0, Reg8::L),
            0x46 => self.bit_hl(0, interconnect),
            0x47 => self.bit(0, Reg8::A),

            0x48 => self.bit(1, Reg8::B),
            0x49 => self.bit(1, Reg8::C),
            0x4A => self.bit(1, Reg8::D),
            0x4B => self.bit(1, Reg8::E),
            0x4C => self.bit(1, Reg8::H),
            0x4D => self.bit(1, Reg8::L),
            0x4E => self.bit_hl(1, interconnect),
            0x4F => self.bit(1, Reg8::A),

            0x50 => self.bit(2, Reg8::B),
            0x51 => self.bit(2, Reg8::C),
            0x52 => self.bit(2, Reg8::D),
            0x53 => self.bit(2, Reg8::E),
            0x54 => self.bit(2, Reg8::H),
            0x55 => self.bit(2, Reg8::L),
            0x56 => self.bit_hl(2, interconnect),
            0x57 => self.bit(2, Reg8::A),

            0x6F => self.bit(5, Reg8::A),

            0x70 => self.bit(6, Reg8::B),

            0x7C => self.bit(7, Reg8::H),
            0x7E => self.bit_hl(7, interconnect),

            0x80 => self.res(0, Reg8::B),
            0x81 => self.res(0, Reg8::C),
            0x82 => self.res(0, Reg8::D),
            0x83 => self.res(0, Reg8::E),
            0x84 => self.res(0, Reg8::H),
            0x85 => self.res(0, Reg8::L),
            0x86 => self.res_hl(0, interconnect),
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

            0xBE => self.res_hl(7, interconnect),

            0xCF => self.set(1, Reg8::A),

            _ => panic!("Unknown CB op: {:#X} at addr: {:#X}", op, pc - 1),
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
