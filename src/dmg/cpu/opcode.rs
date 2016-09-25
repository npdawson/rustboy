#[derive(Copy, Clone)]
pub enum Opcode {
    Ld(Operand8, Operand8),
    Ld16(Reg16, u16),
    LdSpHl,
    Push(Reg16),
    Pop(Reg16),
    Add(Operand8),
    Adc(Operand8),
    Sub(Operand8),
    Sbc(Operand8),
    And(Operand8),
    Xor(Operand8),
    Or(Operand8),
    Cp(Operand8),
    Inc(Operand8),
    Dec(Operand8),
    Daa,
    Cpl,
    AddHl(Reg16),
    Inc16(Reg16),
    Dec16(Reg16),
    AddSp(i8),
    LdHlSp(i8),
    Rlca,
    Rla,
    Rrca,
    Rra,
    Rlc(Operand8),
    Rl(Operand8),
    Rrc(Operand8),
    Rr(Operand8),
    Sla(Operand8),
    Swap(Operand8),
    Sra(Operand8),
    Srl(Operand8),
    Bit(u8, Operand8),
    Set(u8, Operand8),
    Res(u8, Operand8),
    Ccf,
    Scf,
    Nop,
    Halt,
    Stop,
    Di,
    Ei,
    Jmp(JF, u16),
    JmpHl,
    Jr(JF, i8),
    Call(JF, u16),
    Ret(JF),
    Reti,
    Rst(u8),
    Undefined(u8)
}

#[derive(Copy, Clone)]
pub enum Operand8 {
    Register(Reg8),
    Immediate(u8),
    Memory(Addr)
}

#[derive(Copy, Clone)]
pub enum Addr {
    BC,
    DE,
    HL,
    HLD,
    HLI,
    FF_C,
    Imm(u16),
    FF(u8)
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
