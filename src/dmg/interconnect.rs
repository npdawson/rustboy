use byteorder::{LittleEndian, ByteOrder};

use dmg::{Ppu}; // TODO Audio, ...?
use dmg::mem_map::{self, Addr};

const RAM_SIZE: usize = 0x2000;

#[derive(Debug)]
pub struct Interconnect {
    ppu: Ppu,

    in_bootrom: bool,
    boot: Box<[u8]>,
    cart: Box<[u8]>,

    xram: Box<[u8]>,

    ram: Box<[u8]>,
    hram: Box<[u8]>,

    io_regs: Box<[u8]>, // TODO separate into other modules
    iflags: u8,
    dma_addr: u8,

    ie_reg: u8 // Interrupts Enable Register
}

impl Interconnect {
    pub fn new(boot_rom: Box<[u8]>, cart_rom: Box<[u8]>) -> Interconnect {
        Interconnect {
            ppu: Ppu::new(),

            in_bootrom: true,
            boot: boot_rom,
            cart: cart_rom, // TODO move into cart module

            xram: vec![0; RAM_SIZE].into_boxed_slice(), // TODO move into cart mod

            ram: vec![0; RAM_SIZE].into_boxed_slice(),
            hram: vec![0; 128].into_boxed_slice(),

            io_regs: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF00
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x80, 0xBF, 0xF3, 0x00, 0xBF, 0x00, 0x3F, 0x00, //0xFF10
                          0x00, 0xBF, 0x7F, 0xFF, 0x9F, 0x00, 0xBF, 0x00,
                          0xFF, 0x00, 0x00, 0xBF, 0x77, 0xF3, 0xF1, 0x00, //0xFF20
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF30
                          0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
                .into_boxed_slice(),
            iflags: 0,
            dma_addr: 0,

            ie_reg: 0x00,
        }
    }

    // pub fn ppu(&self) -> &Ppu {
    //     &self.ppu
    // }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match mem_map::map_addr(addr) {
            Addr::Rom(offset) => if self.in_bootrom && offset < 0x100 {
                self.boot[offset]
            } else {
                self.cart[offset]
            },
            Addr::Vram(offset) => self.ppu.read_vram(offset),
            Addr::Xram(offset) => self.xram[offset],
            Addr::Ram(offset) => self.ram[offset],
            Addr::Echo(offset) => self.ram[offset],
            Addr::Oam(offset) => self.ppu.read_oam(offset),
            Addr::Hram(offset) => self.hram[offset],

            Addr::InterruptFlags => {0}, //TODO
            Addr::PpuControlReg => self.ppu.read_lcd_ctrl(),
            Addr::PpuStatusReg => self.ppu.read_lcd_stat(),
            Addr::PpuScrollY => self.ppu.scy,
            Addr::PpuScrollX => self.ppu.scx,
            Addr::PpuLcdY => self.ppu.line,
            Addr::PpuLcdYCompare => self.ppu.lyc,
            Addr::PpuOamDma => self.dma_addr,
            Addr::PpuBgPalette => self.ppu.bgp,    // TODO write only?
            Addr::PpuObj0Palette => self.ppu.obp0, // TODO write only?
            Addr::PpuObj1Palette => self.ppu.obp1, // TODO write only?
            Addr::PpuWindowY => self.ppu.wy,
            Addr::PpuWindowX => self.ppu.wx,

            Addr::BootromDisable => if self.in_bootrom { 1 } else { 0 },
            // Addr::CgbRamBank => self.cgb_ram_bank,
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        match mem_map::map_addr(addr) {
            Addr::Rom(offset) => if self.in_bootrom && offset < 0x100 {
                LittleEndian::read_u16(&self.boot[offset..])
            } else {
                LittleEndian::read_u16(&self.cart[offset..])
            },
            Addr::Vram(offset) => self.ppu.read_vram16(offset),
            Addr::Xram(offset) =>
                LittleEndian::read_u16(&self.xram[offset..]),
            Addr::Ram(offset) =>
                LittleEndian::read_u16(&self.ram[offset..]),
            Addr::Echo(offset) =>
                LittleEndian::read_u16(&self.ram[offset..]),
            Addr::Oam(offset) => self.ppu.read_oam16(offset),
            Addr::Hram(offset) =>
                LittleEndian::read_u16(&self.hram[offset..]),

            _ => panic!("Reading 16 bits from IO Regs not supported!")
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match mem_map::map_addr(addr) {
            // TODO send offset and value to cart struct thru function
            Addr::Rom(offset) => panic!("Talking to MBC not yet implemented"),
            Addr::Vram(offset) => self.ppu.write_vram(offset, value),
            Addr::Xram(offset) => self.xram[offset] = value,
            Addr::Ram(offset) => self.ram[offset] = value,
            Addr::Echo(offset) => self.ram[offset] = value,
            Addr::Oam(offset) => self.ppu.write_oam(offset, value),
            Addr::Hram(offset) => self.hram[offset] = value,

            Addr::InterruptFlags => {}, //TODO
            Addr::PpuControlReg => self.ppu.write_lcd_ctrl(value),
            Addr::PpuStatusReg => self.ppu.write_lcd_stat(value),
            Addr::PpuScrollY => self.ppu.scy = value,
            Addr::PpuScrollX => self.ppu.scx = value,
            Addr::PpuLcdY => panic!("Can't change current scanline!"),
            Addr::PpuLcdYCompare => self.ppu.lyc = value,
            Addr::PpuOamDma => {
                self.dma_addr = value;
                self.dma();
            }
            Addr::PpuBgPalette => self.ppu.bgp = value,
            Addr::PpuObj0Palette => self.ppu.obp0 = value,
            Addr::PpuObj1Palette => self.ppu.obp1 = value,
            Addr::PpuWindowY => self.ppu.wy = value,
            Addr::PpuWindowX => self.ppu.wx = value,

            Addr::BootromDisable => self.in_bootrom = value == 0,
        }
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        match mem_map::map_addr(addr) {
            // TODO send offset and value to cart struct thru function
            Addr::Rom(offset) => panic!("Talking to MBC not yet implemented"),
            Addr::Vram(offset) => self.ppu.write_vram16(offset, value),
            Addr::Xram(offset) =>
                LittleEndian::write_u16(&mut self.xram[offset..], value),
            Addr::Ram(offset) =>
                LittleEndian::write_u16(&mut self.ram[offset..], value),
            Addr::Echo(offset) =>
                LittleEndian::write_u16(&mut self.ram[offset..], value),
            Addr::Oam(offset) => self.ppu.write_oam16(offset, value),
            Addr::Hram(offset) =>
                LittleEndian::write_u16(&mut self.hram[offset..], value),

            _ => panic!("tried writing word to unrecognized address {:#x}", addr)
        }
    }

    pub fn step_ppu(&mut self, cycles: usize) {
        if self.ppu.line == 144 {
            self.iflags |= 1 << 0;
        }
        if self.ppu.line == self.ppu.lyc {
            self.iflags |= 1 << 1;
        }
        self.ppu.step(cycles);
    }

    fn dma(&mut self) {
        let addr = (self.dma_addr as u16) << 8;
        let slice = match mem_map::map_addr(addr) {
            Addr::Rom(offset) => &self.cart[offset..],
            Addr::Ram(offset) => &self.ram[offset..],
            Addr::Vram(offset) => &self.ppu.vram[offset..],
            Addr::Xram(offset) => &self.xram[offset..],
            Addr::Echo(offset) => &self.ram[offset..],
            _ => panic!("Can't DMA from addresses higher than 0xF100")
        };
        self.ppu.oam = slice.to_vec().into_boxed_slice();
    }
}
