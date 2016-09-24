use byteorder::{LittleEndian, ByteOrder};

#[derive(Debug)]
pub struct Cart {
    header: Header,
    curr_rom_bank: u8,
    curr_ram_bank: u8,

    pub rom: Box<[u8]>,
    pub ram: Box<[u8]>
}

impl Cart {
    pub fn new(rom: Box<[u8]>) -> Cart{
        let header = Header::new(&rom);
        let ram_size = header.ram_size as usize;
        Cart {
            header: header,
            curr_rom_bank: 0,
            curr_ram_bank: 0,

            rom: rom,
            ram: vec![0; ram_size * 0x2000].into_boxed_slice(),
        }
    }

    pub fn rom_read_byte(&self, offset: usize) -> u8 {
        self.rom[offset]
    }

    pub fn rom_read_word(&self, offset: usize) -> u16 {
        LittleEndian::read_u16(&self.rom[offset..])
    }

    pub fn ram_read_byte(&self, offset: usize) -> u8 {
        self.ram[offset]
    }

    pub fn ram_read_word(&self, offset: usize) -> u16 {
        LittleEndian::read_u16(&self.ram[offset..])
    }

    pub fn ram_write_byte(&mut self, offset: usize, value: u8) {
        self.ram[offset] = value;
    }

    pub fn ram_write_word(&mut self, offset: usize, value: u16) {
        LittleEndian::write_u16(&mut self.ram[offset..], value);
    }

    pub fn mbc_write_byte(&mut self, offset: usize, value: u8) {
        // TODO check cart type and change banks, etc as needed
    }
}

#[derive(Debug)]
struct Header {
    title: &'static str,
    cgb_flag: CgbFlag,
    sgb_flag: SgbFlag,
    cart_type: Mbc,
    rom_size: u8, // size in banks
    ram_size: u8, // size in banks
}

impl Header {
    pub fn new(rom: &Box<[u8]>) -> Header {
        // TODO read this data from the rom
        Header {
            title: "",
            cgb_flag: CgbFlag::No,
            sgb_flag: SgbFlag::No,
            cart_type: Mbc::None,
            rom_size: 2,
            ram_size: 0
        }
    }
}

#[derive(Debug)]
enum Mbc {
    None,
    Mbc1,
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
