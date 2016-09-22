use std::fmt;
use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};
use dmg::gpu::Gpu;

const RAM_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;
const ROM_START: usize = 0x0000;
const ROM_END: usize = 2 * ROM_BANK_SIZE - 1;
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
const OAM_END: usize = OAM_START + OAM_SIZE - 1;
const IOREG_START: usize = 0xFF00;
const IOREG_SIZE: usize = 0x80;
const IOREG_END: usize = IOREG_START + IOREG_SIZE - 1;
const HRAM_START: usize = 0xFF80;
const HRAM_SIZE: usize = 0x007F;
const HRAM_END: usize = HRAM_START + HRAM_SIZE - 1;
const IEREG: usize = 0xFFFF;

pub struct Mmu {
    in_bootrom: bool,
    bootrom: Vec<u8>,
    // switchable banks needs to be implemented
    rom: Vec<u8>,
    cart_type: CartType,
    rom_size: u8,
    ram_size: u8,
    rom_bank: u8,
    //rom0: Box<[u8; ROM_BANK_SIZE]>,
    //rom1: Box<[u8; ROM_BANK_SIZE]>,
    ram: Vec<u8>,  // working ram, half is a switchable bank CGB only
    // vram: Vec<u8>, // video ram, switchable bank 0/1 CGB only
    xram: Vec<u8>, // cart ram, switchable bank
    hram: Vec<u8>,      // high ram
    io_regs: Vec<u8>,
    ie_reg: u8, // Interrupts Enable Register
    gpu: Gpu,
}

impl Mmu {
    pub fn new(boot: Vec<u8>) -> Mmu {
        let cart_type = cart_type(boot[0x147]);
        let rom_size = boot[0x148];
        let ram_size = boot[0x149];
        Mmu {
            in_bootrom: false,
            bootrom: vec![0; 256],
            rom: boot,
            cart_type: cart_type,
            rom_size: rom_size,
            ram_size: ram_size,
            rom_bank: 0,
            ram: vec![0; RAM_SIZE],
            // vram: vec![0; RAM_SIZE],
            xram: vec![0; RAM_SIZE],
            hram: vec![0; 128],
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
            ie_reg: 0x00,
            gpu: Gpu::new()
        }
    }

    pub fn read_byte(&self, addr: usize) -> u8 {
        if addr <= ROM_END {
            if self.in_bootrom && addr < 0x100 {
                self.bootrom[addr]
            } else {
                self.rom_rb(addr)
            }
        } else if addr <= VRAM_END {
            self.gpu.read_vram(addr)
        } else if addr <= XRAM_END {
            self.xram[addr - XRAM_START]
        } else if addr <= ECHO_END {
            if addr <= WRAM_END {
                self.ram[addr - WRAM_START]
            } else {
                self.ram[addr - ECHO_START]
            }
        } else if addr <= OAM_END {
            self.gpu.read_oam(addr)
        } else if addr < IOREG_START {
            //TODO
            panic!("Unused address space: {:#X}", addr);
        } else if addr <= IOREG_END {
            self.read_ioreg(addr)
        } else if addr <= HRAM_END  {
            self.hram[addr - HRAM_START]
        } else {
            self.ie_reg
        }
    }

    pub fn read_word(&self, addr: usize) -> u16 {
        if addr <= ROM_END {
            if self.in_bootrom && addr < 0x100 {
                LittleEndian::read_u16(&self.bootrom[addr..])
            } else {
                self.rom_rw(addr)
            }
        } else if addr <= VRAM_END {
            self.gpu.read_vram16(addr)
        } else if addr <= XRAM_END {
            LittleEndian::read_u16(&self.xram[addr - XRAM_START..])
        } else if addr <= ECHO_END {
            if addr <= WRAM_END {
               LittleEndian::read_u16(&self.ram[addr - WRAM_START..])
            } else {
               LittleEndian::read_u16(&self.ram[addr - ECHO_START..])
            }
        } else if addr <= OAM_END {
            self.gpu.read_oam16(addr)
        } else if addr < IOREG_START {
            //TODO
            panic!("Unused address space: {:#X}", addr);
        } else if addr <= IOREG_END {
            let value = self.read_ioreg16(addr);
            value
        } else if addr <= HRAM_END  {
            LittleEndian::read_u16(&self.hram[addr - HRAM_START..])
        } else {
            self.ie_reg as u16
        }
    }

    pub fn write_byte(&mut self, addr: usize, value: u8) {
        match addr {
            ROM_START ... ROM_END => self.rom_wb(addr, value),
            VRAM_START ... VRAM_END => self.gpu.write_vram(value, addr),
            XRAM_START ... XRAM_END => self.xram[addr - XRAM_START] = value,
            WRAM_START ... WRAM_END => self.ram[addr - WRAM_START] = value,
            ECHO_START ... ECHO_END => self.ram[addr - ECHO_START] = value,
            OAM_START ... OAM_END => self.gpu.write_oam(addr, value),
            IOREG_START ... IOREG_END => self.write_ioreg(value, addr),
            HRAM_START ... HRAM_END => self.hram[addr - HRAM_START] = value,
            IEREG => self.ie_reg = value,
            OAM_END ... 0xFEFF => println!("Why you writing to FEFF, Tetris?"),
            _ => panic!("Failed writing {:#X} to addr {:#X}", value, addr)
        }
    }

    pub fn write_word(&mut self, addr: usize, value: u16) {
        match addr {
            ROM_START ... ROM_END => self.rom_ww(addr, value),
            VRAM_START ... VRAM_END => {
                self.gpu.write_vram16(value, addr);
            }
            XRAM_START ... XRAM_END => {
                LittleEndian::write_u16(&mut self.xram[addr - XRAM_START..], value);
            }
            WRAM_START ... WRAM_END => {
                LittleEndian::write_u16(&mut self.ram[addr - WRAM_START..], value);
            }
            ECHO_START ... ECHO_END => {
                LittleEndian::write_u16(&mut self.ram[addr - WRAM_START..], value);
            }
            OAM_START ... OAM_END => {
                self.gpu.write_oam16(addr, value);
            }
            IOREG_START ... IOREG_END => {
                self.write_ioreg16(value, addr);
            }
            HRAM_START ... HRAM_END => {
                LittleEndian::write_u16(&mut self.hram[addr - HRAM_START..], value);
            }
            IEREG => panic!("Tried to write 16 bits to 8 bit IEREG!"),
            _ => panic!("Failed writing {:#X} to addr {:#X}", value, addr)
        }
    }

    pub fn step_gpu(&mut self, last_t: usize) {
        self.gpu.step(last_t);
        if self.gpu.line == 144 {
            self.write_byte(0xFF0F, 1);
        }
    }

    fn rom_rb(&self, addr: usize) -> u8 {
        match addr {
            0x0000 ... 0x3FFF => self.rom[addr],
            _ => match self.rom_bank {
                0 | 1 => self.rom[addr],
                _ => self.rom[addr + ROM_BANK_SIZE * ((self.rom_bank as usize) - 1)]
            }
        }
    }

    fn rom_rw(&self, addr: usize) -> u16 {
        match addr {
            0x0000 ... 0x3FFF => LittleEndian::read_u16(&self.rom[addr..]),
            _ => match self.rom_bank {
                0 | 1 => LittleEndian::read_u16(&self.rom[addr..]),
                _ => LittleEndian::read_u16(&self.rom[(addr
                                                       + ROM_BANK_SIZE
                                                       * ((self.rom_bank as usize) - 1))..])
            }
        }
    }

    fn rom_wb(&mut self, addr: usize, value: u8) {
        match addr {
            0x2000 ... 0x3FFF => self.rom_bank = value,
            _ => panic!("writing to MBC at {:#X} with {:#X}", addr, value)
        }
    }

    fn rom_ww(&mut self, addr: usize, value: u16) {
        panic!("writing {:#X} to {:#X}", value, addr);
    }

    fn dma(&mut self, value: u8) {
        let addr = (value as usize) << 8;
        let slice = match addr {
            0x0000 ... 0x3FFF => &self.rom[addr..(addr + OAM_SIZE)],
            0x4000 ... 0x7FFF => {
                let addr = if self.rom_bank != 0 {
                    addr + (ROM_BANK_SIZE * ((self.rom_bank as usize) - 1))
                } else {
                    addr
                };
                &self.rom[addr..(addr + OAM_SIZE)]
            }
            VRAM_START ... VRAM_END => {
                let addr = addr - VRAM_START;
                &self.gpu.vram[addr..(addr + OAM_SIZE)]
            }
            XRAM_START ... XRAM_END => {
                let addr = addr - XRAM_START;
                &self.xram[addr..(addr + OAM_SIZE)]
            }
            WRAM_START ... WRAM_END => {
                let addr = addr - WRAM_START;
                &self.ram[addr..(addr + OAM_SIZE)]
            }
            ECHO_START ... 0xF100 => {
                let addr = addr - ECHO_START;
                &self.ram[(addr)..(addr + OAM_SIZE)]
            }
            _ => panic!("DMA from invalid address: {:#X}", addr)
        };
        self.gpu.oam = slice.to_vec();
    }

    fn read_ioreg(&self, addr: usize) -> u8 {
        if addr & 0xF0 >= 0x40 { // GPU
            self.gpu.read_byte(addr)
        } else {
            self.io_regs[addr - IOREG_START]
        }
    }

    fn read_ioreg16(&self, addr: usize) -> u16 {
        panic!("Reading a word from IORegs not yet supported!")
    }

    fn write_ioreg(&mut self, value: u8, addr: usize) {
        if addr == 0xFF50 {
            self.in_bootrom = false;
        } else if addr == 0xFF46 {
            self.dma(value);
        } else if addr & 0xF0 >= 0x40 { // GPU
            self.gpu.write_byte(value, addr);
        } else {
            self.io_regs[addr - IOREG_START] = value;
        }
    }

    fn write_ioreg16(&mut self, value: u16, addr: usize) {
        panic!("Writing a word to IORegs not yet supported!")
    }
}

impl fmt::Debug for Mmu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(f, "TODO: Impl Debug for MMU")
    }
}

enum CartType {
    ROM = 0x00,

    MBC3_RAM_BAT = 0x13,

    MBC5_RAM_BAT = 0x1B,
}

fn cart_type(value: u8) -> CartType {
    match value {
        0x00 => CartType::ROM,
        0x13 => CartType::MBC3_RAM_BAT,
        0x1B => CartType::MBC5_RAM_BAT,
        _ => panic!("Cart Type {:#X} not yet implemented!", value)
    }
}
