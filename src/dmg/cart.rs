use byteorder::{LittleEndian, ByteOrder};

#[derive(Debug)]
pub struct Cart {
    header: Header,

    ram_timer_enable: bool,
    rom_bank: u8,
    ram_bank_rtc: u8,
    rom_ram_mode: RomRam,

    pub rom: Box<[u8]>,
    pub ram: Box<[u8]>
}

impl Cart {
    pub fn new(rom: Box<[u8]>) -> Cart{
        let header = Header::new(&rom);
        let ram_size = match header.ram_size {
            RamSize::RamNone => 0,
            RamSize::Ram2K => 2048,
            RamSize::Ram8K => 8192,
            RamSize::Ram32K => 32 * 1024,
            RamSize::Ram128K => 128 * 1024,
            RamSize::Ram64K => 64 * 1024
        };
        Cart {
            header: header,

            ram_timer_enable: false,
            rom_bank: 0,
            ram_bank_rtc: 0,
            rom_ram_mode: RomRam::Rom,

            rom: rom,
            ram: vec![0; ram_size].into_boxed_slice(),
        }
    }

    pub fn rom_read_byte(&self, offset: usize) -> u8 {
        match self.header.cart_type {
            Mbc::None => self.rom[offset],
            Mbc::Mbc1 |
            Mbc::Mbc1Ram |
            Mbc::Mbc1RamBat => self.mbc1_rom_read_byte(offset),
            Mbc::Mbc3RamBat => self.mbc1_rom_read_byte(offset),
        }
    }

    pub fn rom_read_word(&self, offset: usize) -> u16 {
        if offset < 0x4000 {
            LittleEndian::read_u16(&self.rom[offset..])
        } else {
            let bank_offset = 0x4000 * (self.rom_bank.saturating_sub(1)) as usize;
            LittleEndian::read_u16(&self.rom[offset+bank_offset..])
        }
    }

    pub fn ram_read_byte(&self, offset: usize) -> u8 {
        if !self.ram_timer_enable { return 0xFF }
        self.ram[offset]
    }

    pub fn ram_read_word(&self, offset: usize) -> u16 {
        if !self.ram_timer_enable { return 0xFFFF }
        LittleEndian::read_u16(&self.ram[offset..])
    }

    pub fn ram_write_byte(&mut self, offset: usize, value: u8) {
        if self.ram_timer_enable {
            self.ram[offset] = value;
        }
    }

    pub fn ram_write_word(&mut self, offset: usize, value: u16) {
        if self.ram_timer_enable {
            LittleEndian::write_u16(&mut self.ram[offset..], value);
        }
    }

    pub fn mbc_write_byte(&mut self, offset: usize, value: u8) {
        let cart_type = self.header.cart_type;
        match cart_type {
            Mbc::None => {},
            Mbc::Mbc1 |
            Mbc::Mbc1Ram |
            Mbc::Mbc1RamBat => self.mbc1_write(offset, value),
            Mbc::Mbc3RamBat => self.mbc3_write(offset, value),
        }
    }

    fn mbc1_write(&mut self, offset: usize, value: u8) {
        match offset {
            0x0000 ... 0x1FFF => self.ram_timer_enable = value & 0xF == 0xA,
            0x2000 ... 0x3FFF => self.rom_bank = value & 0x1F,
            0x4000 ... 0x5FFF => {
                self.rom_bank = self.rom_bank & 0x1F | (value & 0x3) << 5;
                self.ram_bank_rtc = value & 0x3;
            },
            0x6000 ... 0x7FFF => self.rom_ram_mode = match value & 1 {
                0 => RomRam::Rom,
                _ => RomRam::Ram
            },
            _ => unreachable!()
        }
    }

    fn mbc3_write(&mut self, offset: usize, value: u8) {
        match offset {
            0x0000 ... 0x1FFF => self.ram_timer_enable = value & 0xF == 0xA,
            0x2000 ... 0x3FFF => self.rom_bank = value & 0x1F,
            0x4000 ... 0x5FFF => self.ram_bank_rtc = value & 0xF,
            0x6000 ... 0x7FFF => panic!("RTC not yet supported!"),
            _ => unreachable!()
        }
    }

    fn mbc1_rom_read_byte(&self, offset: usize) -> u8 {
        if offset < 0x4000 {
            self.rom[offset]
        } else {
            let bank = match self.header.cart_type {
                Mbc::Mbc1 |
                Mbc::Mbc1Ram |
                Mbc::Mbc1RamBat =>
                    if let RomRam::Ram = self.rom_ram_mode {
                        self.rom_bank & 0b11111
                    } else {
                        self.rom_bank
                    },
                _ => self.rom_bank
            };
            let bank_offset = 0x4000 * (bank.saturating_sub(1)) as usize;
            self.rom[offset + bank_offset]
        }
    }
}

#[derive(Debug)]
struct Header {
    title: Box<[u8]>,
    cgb_flag: CgbFlag,
    sgb_flag: SgbFlag,
    cart_type: Mbc,
    rom_size: RomSize,
    ram_size: RamSize,
}

impl Header {
    pub fn new(rom: &Box<[u8]>) -> Header {
        // TODO read this data from the rom
        let title = rom[0x134..0x143].to_vec().into_boxed_slice();
        let cgb = match rom[0x143] {
            0x80 => CgbFlag::Capable,
            0xC0 => CgbFlag::Only,
            _ => CgbFlag::No
        };
        let sgb = match rom[0x146] {
            0x03 => SgbFlag::Yes,
            _ => SgbFlag::No
        };
        let cart = match rom[0x147] {
            0x00 => Mbc::None,
            0x01 => Mbc::Mbc1,
            0x02 => Mbc::Mbc1Ram,
            0x03 => Mbc::Mbc1RamBat,
            0x13 => Mbc::Mbc3RamBat,
            _ => panic!("MBC type {:#x} not yet supported!", rom[0x147])
        };
        let rom_size = match rom[0x148] {
            0x0 => RomSize::Rom2Banks,
            0x1 => RomSize::Rom4Banks,
            0x2 => RomSize::Rom8Banks,
            0x3 => RomSize::Rom16Banks,
            0x4 => RomSize::Rom32Banks,
            0x5 => RomSize::Rom64Banks,
            0x6 => RomSize::Rom128Banks,
            0x7 => RomSize::Rom256Banks,
            0x8 => RomSize::Rom512Banks,
            _ => panic!("Unknown Rom size in header: {:#x}", rom[0x148])
        };
        let ram_size = match rom[0x149] {
            0 => RamSize::RamNone,
            1 => RamSize::Ram2K,
            2 => RamSize::Ram8K,
            3 => RamSize::Ram32K,
            4 => RamSize::Ram128K,
            5 => RamSize::Ram64K,
            _ => panic!("Unknown Ram size in header: {:#x}", rom[0x149])
        };
        Header {
            title: title,
            cgb_flag: cgb,
            sgb_flag: sgb,
            cart_type: cart,
            rom_size: rom_size,
            ram_size: ram_size
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Mbc {
    None,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBat,
    Mbc3RamBat,
}

#[derive(Debug)]
enum CgbFlag {
    No,
    Capable,
    Only
}

#[derive(Debug)]
enum SgbFlag {
    No,
    Yes
}

#[derive(Debug)]
enum RomSize {
    Rom2Banks,
    Rom4Banks,
    Rom8Banks,
    Rom16Banks,
    Rom32Banks,
    Rom64Banks,
    Rom128Banks,
    Rom256Banks,
    Rom512Banks,
}

#[derive(Debug)]
enum RamSize {
    RamNone,
    Ram2K,
    Ram8K,
    Ram32K,
    Ram128K,
    Ram64K
}

#[derive(Debug)]
enum RomRam {
    Rom,
    Ram
}
