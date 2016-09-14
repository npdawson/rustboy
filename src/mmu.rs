use std::fmt;

const RAM_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;
const ROM_START: usize = 0x0000;
const ROM_END: usize = 0x7FFF;
const VRAM_START: usize = 0x8000;
const VRAM_END: usize = VRAM_START + RAM_SIZE - 1;
const XRAM_START: usize = 0xA000;
const XRAM_END: usize = XRAM_START + RAM_SIZE - 1;
const WRAM_START: usize = 0xC000;
const WRAM_END: usize = WRAM_START + RAM_SIZE - 1;
const ECHO_START: usize = 0xE000;
const ECHO_END: usize = 0xFDFF;
const OAM_START: usize = 0xFE00;
const OAM_SIZE: usize = 0xA0;
const OAM_END: usize = 0xFE9F;
const IOREG_START: usize = 0xFF00;
const IOREG_SIZE: usize = 0x80;
const IOREG_END: usize = 0xFF7F;
const HRAM_START: usize = 0xFF80;
const HRAM_END: usize = 0xFFFE;
const HRAM_SIZE: usize = HRAM_END - HRAM_START;
const IEREG: usize = 0xFFFF;

pub struct Mmu {
    // switchable banks needs to be implemented
    rom: Vec<u8>,
    //rom0: Box<[u8; ROM_BANK_SIZE]>,
    //rom1: Box<[u8; ROM_BANK_SIZE]>,
    ram: Vec<u8>,  // working ram, half is a switchable bank CGB only
    vram: Vec<u8>, // video ram, switchable bank 0/1 CGB only
    xram: Vec<u8>, // cart ram, switchable bank
    hram: Vec<u8>,      // high ram
    io_regs: Vec<u8>,
    oam: Vec<u8>,
    ie_reg: u8, // Interrupts Enable Register
}

impl Mmu {
    pub fn new(rom: Vec<u8>) -> Mmu {
        Mmu {
            rom: rom,
            ram: vec![0; RAM_SIZE],
            vram: vec![0; RAM_SIZE],
            xram: vec![0; RAM_SIZE],
            hram: vec![0; 128],
            oam: vec![0; OAM_SIZE],
            io_regs: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF00
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x80, 0xBF, 0xF3, 0x00, 0xBF, 0x00, 0x3F, 0x00, //0xFF10
                          0x00, 0xBF, 0x7F, 0xFF, 0x9F, 0x00, 0xBF, 0x00,
                          0xFF, 0x00, 0x00, 0xBF, 0x77, 0xF3, 0xF1, 0x00, //0xFF20
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF30
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x91, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFC, //0xFF40
                          0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF50
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF60
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF70
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            ie_reg: 0x00
        }
    }

    pub fn read_byte(&self, addr: usize) -> u8 {
        if addr <= ROM_END {
            self.rom[addr]
        } else if addr <= VRAM_END {
            self.vram[addr - VRAM_START]
        } else if addr <= XRAM_END {
            self.xram[addr - XRAM_START]
        } else if addr <= ECHO_END {
            if addr <= WRAM_END {
                self.ram[addr - WRAM_START]
            } else {
                self.ram[addr - ECHO_START]
            }
        } else if addr <= OAM_END {
            self.oam[addr - OAM_START]
        } else if addr < IOREG_START {
            //TODO
            panic!("Unused address space: {:#X}", addr);
        } else if addr <= IOREG_END {
            self.io_regs[addr - IOREG_START]
        } else if addr <= HRAM_END  {
            self.hram[addr - HRAM_START]
        } else {
            self.ie_reg
        }
    }

    pub fn write_byte(&mut self, value: u8, addr: usize) {
        match addr {
            ROM_START ... ROM_END => println!("Tried writing to ROM!"),
            VRAM_START ... VRAM_END => self.vram[addr - VRAM_START] = value,
            XRAM_START ... XRAM_END => self.xram[addr - XRAM_START] = value,
            WRAM_START ... WRAM_END => self.ram[addr - WRAM_START] = value,
            ECHO_START ... ECHO_END => self.ram[addr - ECHO_START] = value,
            OAM_START ... OAM_END => self.oam[addr - OAM_START] = value,
            IOREG_START ... IOREG_END => self.io_regs[addr - IOREG_START] = value,
            HRAM_START ... HRAM_END => self.hram[addr - HRAM_START] = value,
            IEREG => self.ie_reg = value,
            _ => panic!("Failed writing {:#X} to addr {:#X}", value, addr)
        }
    }
}

impl fmt::Debug for Mmu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "TODO: Impl Debug for MMU")
    }
}
