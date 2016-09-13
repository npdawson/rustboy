#[derive(Default, Debug)]
pub struct Cpu {
    reg_pc: u16,
    reg_sp: u16,
    reg_a: u8,
    reg_f: u8, // separate flags?
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,
    // clock time of last instruction
    last_m: u8,
    last_t: u8,
    // clock time total
    clock_m: u32,
    clock_t: u32,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu::default()
    }
}
