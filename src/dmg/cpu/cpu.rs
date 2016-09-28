use dmg::Interconnect;
use super::opcode::{Opcode, Operand8, Addr, Reg8, Reg16, JF};
use super::opcode::Opcode::*;
use super::opcode::Operand8::*;
use super::opcode::Reg8::*;
use super::opcode::Reg16::*;
use super::Instruction;

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
    ime_next_cycle: bool,
    // halted, waiting for interrupt
    pub halted: bool,
    // stopped, waiting for button press
    pub stopped: bool,
    // clock time of last instruction
    last_m: usize,
    // clock time total
    clock_m: usize,
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
            ime_next_cycle: false,
            halted: false,
            stopped: false,
            // clock time of last instruction
            last_m: 0,
            // clock time total
            clock_m: 0,
        }
    }

    pub fn current_pc(&self) -> u16 {
        self.reg_pc
    }

    pub fn step(&mut self, interconnect: &mut Interconnect) -> usize {
        if self.halted || self.stopped {
            1 // wait for interrupt/button press
        } else {
            if self.ime_next_cycle {
                self.ime_next_cycle = false;
                self.ime = true;
            }
            let pc = self.reg_pc;
            // print!("{:#x}: ", pc);
            let instr = Instruction::fetch(pc, interconnect);
            // println!("{:?}", instr.opcode());
            self.reg_pc = pc.saturating_add(instr.bytes() as u16);

            let cycles = self.execute(instr, interconnect);

            self.last_m = cycles;
            self.clock_m += cycles;
            cycles
        }
    }

    fn execute(&mut self, instr: Instruction, interconnect: &mut Interconnect) -> usize {
        let opcode = instr.opcode();
        let mut extra_jump_cycles = 0;
        match opcode {
            Ld(op1, op2) => self.ld(op1, op2, interconnect),
            Ld16(reg, imm) => self.write_reg16(reg, imm),
            LdnnSp(imm) => self.ld_nn_sp(imm, interconnect),
            LdSpHl => self.reg_sp = self.read_reg16(HL),
            Push(reg) => self.push(reg, interconnect),
            Pop(reg) => self.pop(reg, interconnect),
            Add(op) => self.add(op, interconnect),
            Adc(op) => self.adc(op, interconnect),
            Sub(op) => self.sub(op, interconnect),
            Sbc(op) => self.sbc(op, interconnect),
            And(op) => self.and(op, interconnect),
            Xor(op) => self.xor(op, interconnect),
            Or(op) => self.or(op, interconnect),
            Cp(op) => self.cp(op, interconnect),
            Inc(op) => self.inc(op, interconnect),
            Dec(op) => self.dec(op, interconnect),
            Daa => self.daa(),
            Cpl => self.cpl(),
            AddHl(reg) => self.add_hl(reg),
            Inc16(reg) => self.inc16(reg),
            Dec16(reg) => self.dec16(reg),
            AddSp(r8) => self.add_sp(r8),
            LdHlSp(r8) => self.ld_hl_sp(r8),
            Rlca |
            Rla |
            Rrca |
            Rra => self.rot(Reg(A), opcode, interconnect),
            Rlc(op) |
            Rl(op) |
            Rrc(op) |
            Rr(op) => self.rot(op, opcode, interconnect),
            Swap(op) => self.swap(op, interconnect),
            Sla(op) |
            Sra(op) |
            Srl(op) => self.shift(op, opcode, interconnect),
            Bit(bit, op) => self.bit(bit, op, interconnect),
            Set(bit, op) => self.set(bit, op, interconnect),
            Res(bit, op) => self.res(bit, op, interconnect),
            Ccf => self.ccf(),
            Scf => self.scf(),
            Nop => {},
            Halt => self.halt(interconnect),
            Stop => self.stop(),
            Di => self.ime = false,
            Ei => self.ime_next_cycle = true,
            Jmp(flag, addr) => { extra_jump_cycles = self.jmp(flag, addr); },
            JmpHl => self.jmp_hl(),
            Jr(flag, r8) => { extra_jump_cycles = self.jr(flag, r8); },
            Call(flag, addr) =>
            { extra_jump_cycles = self.call(flag, addr, interconnect); },
            Ret(flag) => { extra_jump_cycles = self.ret(flag, interconnect); },
            Reti => {
                self.ime = true;
                self.ret(JF::Always, interconnect);
            },
            Rst(addr) => self.rst(addr, interconnect),
            Undefined(op) => panic!("Undefined opcode: {:#x}", op)
        }
        (instr.cycles() as usize) + extra_jump_cycles
    }

    pub fn interrupt(&mut self, addr: u16, interconnect: &mut Interconnect) -> usize {
        self.ime = false;
        let cycles = 4;
        self.last_m = cycles;
        self.clock_m += cycles;
        self.push(PC, interconnect);
        self.reg_pc = addr;
        cycles
    }

    fn add(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        let result = old.wrapping_add(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = (old & 0xF) + (value & 0xF) >= 0x10;
        self.flag_reg.carry = (old as u16) + (value as u16) > 0xFF;
    }

    fn add_sp(&mut self, r8: i8) {
        let sp = self.reg_sp;
        let imm = r8 as u16;
        self.reg_sp = sp.wrapping_add(imm);
        self.flag_reg.zero = false;
        self.flag_reg.sub = false;
        self.flag_reg.half = (sp & 0xF) + (imm & 0xF) & 0x10 == 0x10;
        self.flag_reg.carry = (sp & 0xFF) + (imm & 0xFF) & 0x100 == 0x100;
    }

    fn adc(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let mut value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        if self.flag_reg.carry {
            value = value.wrapping_add(1);
        }
        let result = old.wrapping_add(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half =
            (old & 0xF) + (value & 0xF) >= 0x10;
        self.flag_reg.carry = (old as u16) + (value as u16) >= 0x100;
    }

    fn add_hl(&mut self, reg: Reg16) {
        let old = self.read_reg16(HL);
        let value = self.read_reg16(reg);
        let result = old.wrapping_add(value);
        self.write_reg16(HL, result);
        self.flag_reg.sub = false;
        self.flag_reg.half =
            (old & 0xFFF).wrapping_add(value & 0xFFF) >= 0x1000;
        self.flag_reg.carry = (old as u32).wrapping_add(value as u32) >= 0x10000;
    }

    fn and(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        let result = old & value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = true;
        self.flag_reg.carry = false;
    }

    fn bit(&mut self, bit: u8, op: Operand8, interconnect: &mut Interconnect) {
        let value = match op {
            Reg(reg) => self.read_reg(reg),
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            }
            _ => unreachable!()
        };
        self.flag_reg.zero = value & (1 << bit) == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = true;
    }

    fn call (&mut self, flag: JF, addr: u16, interconnect: &mut Interconnect) -> usize {
        let jump = self.jump_match(flag);
        if jump {
            self.push(PC, interconnect);
            self.reg_pc = addr;
            return 3;
        }
        0
    }

    fn ccf(&mut self) {
        // toggle carry, NOT CLEAR
        self.flag_reg.carry = !self.flag_reg.carry;
        self.flag_reg.half = false;
        self.flag_reg.sub = false;
    }

    fn cp(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        let result = old.wrapping_sub(value);
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value & 0x0F;
        self.flag_reg.carry = old < value;
    }

    fn cpl(&mut self) {
        // toggle all bits of A
        let old = self.reg_a;
        self.reg_a = old ^ 0xFF;
        self.flag_reg.sub = true;
        self.flag_reg.half = true;
    }

    fn daa(&mut self) {
        // decimal adjust accumulator
        let old = self.reg_a;
        let mut value = 0u8;
        if !self.flag_reg.sub {
            if self.flag_reg.half || (old & 0xF) > 9 {
                value = value.wrapping_add(0x06);
            }
            if self.flag_reg.carry || (old > 0x9F) {
                value = value.wrapping_add(0x60);
            }
        } else {
            if self.flag_reg.half {
                value = value.wrapping_sub(0x06);
            }
            if self.flag_reg.carry {
                value = value.wrapping_sub(0x60);
            }
        };
        let result = old.wrapping_add(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.half = false;
        self.flag_reg.carry =
            (old as u16).wrapping_add(value as u16) >= 0x100;
    }

    fn dec(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old;
        let result;
        match op {
            Reg(reg) => {
                old = self.read_reg(reg);
                result = old.wrapping_sub(1);
                self.write_reg(reg, result);
            },
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                old = interconnect.read_byte(addr);
                result = old.wrapping_sub(1);
                interconnect.write_byte(addr, result);
            },
            _ => unreachable!()
        };
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0xF == 0;
    }

    fn dec16(&mut self, reg: Reg16) {
        let old = self.read_reg16(reg);
        self.write_reg16(reg, old.wrapping_sub(1));
    }

    fn halt(&mut self, interconnect: &mut Interconnect) {
        // TODO halt bug
        self.halted = true;
    }

    fn inc(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old;
        let result;
        match op {
            Reg(reg) => {
                old = self.read_reg(reg);
                result = old.wrapping_add(1);
                self.write_reg(reg, result);
            },
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                old = interconnect.read_byte(addr);
                result = old.wrapping_add(1);
                interconnect.write_byte(addr, result);
            },
            _ => unreachable!()
        };
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = (old & 0xF).wrapping_add(1) == 0x10;
    }

    fn inc16(&mut self, reg: Reg16) {
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

    fn jr(&mut self, flag: JF, rel_addr: i8)  -> usize {
        let jump = self.jump_match(flag);
        let jump_addr = self.reg_pc.wrapping_add(rel_addr as u16);
        if jump {
            self.reg_pc = jump_addr;
            return 1;
        }
        0
    }

    fn jmp(&mut self, flag: JF, addr: u16) -> usize {
        let jump = self.jump_match(flag);
        if jump {
            self.reg_pc = addr;
            return 1;
        }
        0
    }

    fn jmp_hl(&mut self) {
        let addr = self.read_reg16(HL);
        self.reg_pc = addr;
    }

    fn ld(&mut self, op1: Operand8, op2: Operand8, interconnect: &mut Interconnect) {
        let value = match op2 {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(mem) => {
                let addr = match mem {
                    Addr::BC => self.read_reg16(BC),
                    Addr::DE => self.read_reg16(DE),
                    Addr::HL  |
                    Addr::HLD |
                    Addr::HLI => self.read_reg16(HL),
                    Addr::FF_C => 0xFF00 | self.reg_c as u16,
                    Addr::Imm(imm) => imm,
                    Addr::FF(imm) => 0xFF00 | imm as u16,
                };
                if let Addr::HLD = mem {
                    self.write_reg16(HL, addr.wrapping_sub(1));
                } else if let Addr::HLI = mem {
                    self.write_reg16(HL, addr.wrapping_add(1));
                }
                interconnect.read_byte(addr)
            },
        };
        match op1 {
            Reg(reg) => self.write_reg(reg, value),
            Mem(mem) => {
                let addr = match mem {
                    Addr::BC => self.read_reg16(BC),
                    Addr::DE => self.read_reg16(DE),
                    Addr::HL  |
                    Addr::HLD |
                    Addr::HLI => self.read_reg16(HL),
                    Addr::FF_C => 0xFF00 | self.reg_c as u16,
                    Addr::Imm(imm) => imm,
                    Addr::FF(imm) => 0xFF00 | imm as u16,
                };
                interconnect.write_byte(addr, value);
                if let Addr::HLD = mem {
                    self.write_reg16(HL, addr.wrapping_sub(1));
                } else if let Addr::HLI = mem {
                    self.write_reg16(HL, addr.wrapping_add(1));
                }
            }
            _ => unreachable!()
        }
    }

    fn ld_hl_sp(&mut self, r8: i8) {
        let sp = self.reg_sp;
        let offset = r8 as u16;
        let result = sp.wrapping_add(offset);
        self.write_reg16(HL, result);
        self.flag_reg.zero = false;
        self.flag_reg.sub = false;
        self.flag_reg.half =
            (sp & 0xF).wrapping_add(offset & 0xF) >= 0x10;
        self.flag_reg.carry = (sp & 0xFF) + (offset & 0xFF) >= 0x100;
    }

    fn ld_nn_sp(&mut self, addr: u16, interconnect: &mut Interconnect) {
        let sp = self.reg_sp;
        interconnect.write_word(addr, sp);
    }

    fn or(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        let result = old | value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn pop(&mut self, reg: Reg16, interconnect: &mut Interconnect) {
        let sp = self.reg_sp;
        let value = interconnect.read_word(sp);
        self.write_reg16(reg, value);
        self.reg_sp += 2;
    }

    fn push(&mut self, reg: Reg16, interconnect: &mut Interconnect) {
        let sp = self.reg_sp - 2;
        let value = self.read_reg16(reg);
        interconnect.write_word(sp, value);
        self.reg_sp = sp;
    }

    fn res(&mut self, bit: u8, op: Operand8, interconnect: &mut Interconnect) {
        let old;
        let result;
        match op {
            Reg(reg) => {
                old = self.read_reg(reg);
                result = old & !(1 << bit);
                self.write_reg(reg, result);
            },
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                old = interconnect.read_byte(addr);
                result = old & !(1 << bit);
                interconnect.write_byte(addr, result);
            }
            _ => unreachable!()
        };
    }

    fn ret(&mut self, flag: JF, interconnect: &mut Interconnect) -> usize {
        let jump = self.jump_match(flag);
        if jump {
            let sp = self.reg_sp;
            let addr = interconnect.read_word(sp);
            self.reg_pc = addr;
            self.reg_sp = sp + 2;
            match flag {
                JF::Always => return 2,
                _ => return 3
            }
        }
        0
    }

    fn rot(&mut self, op: Operand8, opcode: Opcode, interconnect: &mut Interconnect) {
        let addr = self.read_reg16(HL);
        let result;
        let old = match op {
            Reg(reg) => self.read_reg(reg),
            Mem(Addr::HL) => interconnect.read_byte(addr),
            _ => unreachable!()
        };
        let carrybit = self.flag_reg.carry;
        match opcode {
            Rl(_) |
            Rla => {
                result = old << 1 | (if carrybit {1} else {0});
                self.flag_reg.carry = old & (1 << 7) != 0;
            },
            Rlc(_) |
            Rlca => {
                result = old.rotate_left(1);
                self.flag_reg.carry = old >> 7 != 0;
            },
            Rr(_) |
            Rra => {
                result = old >> 1 | (if carrybit {1 << 7} else {0});
                self.flag_reg.carry = old & 1 != 0;
            },
            Rrc(_) |
            Rrca => {
                result = old.rotate_right(1);
                self.flag_reg.carry = old & 1 != 0;
            },
            _ => unreachable!()
        }
        match op {
            Reg(reg) => self.write_reg(reg, result),
            Mem(Addr::HL) => interconnect.write_byte(addr, result),
            _ => unreachable!()
        }
        self.flag_reg.zero = match opcode {
            Rla |
            Rlca |
            Rra |
            Rrca => false,
            _ => result == 0
        };
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
    }

    fn rst(&mut self, addr: u8, interconnect: &mut Interconnect) {
        self.push(PC, interconnect);
        self.reg_pc = addr as u16;
    }

    fn sbc(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let mut value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        if self.flag_reg.carry {
            value = value.wrapping_add(1);
        }
        let result = old.wrapping_sub(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = true;
        self.flag_reg.half = (old & 0xF) < (value & 0xF);
        self.flag_reg.carry = old < value;
    }

    fn scf(&mut self) {
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = true;
    }

    fn set(&mut self, bit: u8, op: Operand8, interconnect: &mut Interconnect) {
        let addr = self.read_reg16(HL);;
        let old = match op {
            Reg(reg) => self.read_reg(reg),
            Mem(Addr::HL) => interconnect.read_byte(addr),
            _ => unreachable!()
        };
        let result = old | (1 << bit);
        match op {
            Reg(reg) => self.write_reg(reg, result),
            Mem(Addr::HL) => interconnect.write_byte(addr, result),
            _ => unreachable!()
        }
    }

    fn shift(&mut self, op: Operand8, opcode: Opcode, interconnect: &mut Interconnect) {
        let result;
        let addr = self.read_reg16(HL);;
        let old = match op {
            Reg(reg) => self.read_reg(reg),
            Mem(Addr::HL) => interconnect.read_byte(addr),
            _ => unreachable!()
        };
        match opcode {
            Sla(_) => {
                result = old << 1;
                self.flag_reg.carry = old & (1 << 7) != 0;
            },
            Sra(_) => {
                let sign = old & (1 << 7);
                result = old >> 1 | sign;
                self.flag_reg.carry = false;
            },
            Srl(_) => {
                result = old >> 1;
                self.flag_reg.carry = old & 1 != 0;
            },
            _ => unreachable!()
        }
        match op {
            Reg(reg) => self.write_reg(reg, result),
            Mem(Addr::HL) => interconnect.write_byte(addr, result),
            _ => unreachable!()
        }
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
    }

    fn stop(&mut self) {
        // self.stopped = true;
    }

    fn sub(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        let result = old.wrapping_sub(value);
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = true;
        self.flag_reg.half = old & 0x0F < value & 0x0F;
        self.flag_reg.carry = old < value;
    }

    fn swap(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let addr = self.read_reg16(HL);
        let old = match op {
            Reg(reg) => self.read_reg(reg),
            Mem(Addr::HL) => interconnect.read_byte(addr),
            _ => unreachable!()
        };
        let lo = old & 0x0F;
        let hi = old & 0xF0;
        let result = lo << 4 | hi >> 4;
        match op {
            Reg(reg) => self.write_reg(reg, result),
            Mem(Addr::HL) => interconnect.write_byte(addr, result),
            _ => unreachable!()
        }
        self.flag_reg.zero = result == 0x00;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
    }

    fn xor(&mut self, op: Operand8, interconnect: &mut Interconnect) {
        let old = self.reg_a;
        let value = match op {
            Reg(reg) => self.read_reg(reg),
            Imm(imm) => imm,
            Mem(Addr::HL) => {
                let addr = self.read_reg16(HL);
                interconnect.read_byte(addr)
            },
            _ => unreachable!()
        };
        let result = old ^ value;
        self.reg_a = result;
        self.flag_reg.zero = result == 0;
        self.flag_reg.sub = false;
        self.flag_reg.half = false;
        self.flag_reg.carry = false;
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
            BC =>
                (self.read_reg(Reg8::B) as u16) << 8 | self.read_reg(Reg8::C) as u16,
            DE =>
                (self.read_reg(Reg8::D) as u16) << 8 | self.read_reg(Reg8::E) as u16,
            HL =>
                (self.read_reg(Reg8::H) as u16) << 8 | self.read_reg(Reg8::L) as u16,
            SP => self.reg_sp,
            PC => self.reg_pc,
            AF =>
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
            BC => {
                self.write_reg(Reg8::B, hi);
                self.write_reg(Reg8::C, lo);
            }
            DE => {
                self.write_reg(Reg8::D, hi);
                self.write_reg(Reg8::E, lo);
            }
            HL => {
                self.write_reg(Reg8::H, hi);
                self.write_reg(Reg8::L, lo);
            }
            SP => self.reg_sp = value,
            PC => self.reg_pc = value,
            AF => {
                self.write_reg(Reg8::A, hi);
                self.write_reg(Reg8::F, lo);
            }
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
