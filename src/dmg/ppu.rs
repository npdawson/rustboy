// const FB_SIZE: usize = 160*144;
// const WHITE: [u8; 4] = [255, 255, 255, 255];
// const LGRAY: [u8; 4] = [192, 192, 192, 255];
// const DGRAY: [u8; 4] = [ 96,  96,  96, 255];
// const BLACK: [u8; 4] = [  0,   0,   0, 255];

use byteorder::{ByteOrder, LittleEndian};

#[derive(Debug)]
pub struct Ppu {
    pub vram: Box<[u8]>,
    pub oam: Box<[u8]>,

    // fb: Box<[u32]>,
    mode: Mode,
    modeclock: usize,
    pub line: u8, // LY: 160 lines
    // LY Compare
    pub lyc: u8,
    // LCD CTRL, make separate struct?
    switchbg: bool,
    bgmap: bool,
    bgtile: bool,
    switchlcd: bool,
    // LCD STAT, make separate struct?
    coincidence_int: bool,
    mode2oam_int: bool,
    mode1vblank_int: bool,
    mode0hblank_int: bool,
    // Scroll coords
    pub scy: u8,
    pub scx: u8,
    // Window coords
    pub wy: u8,
    pub wx: u8,
    // background palette
    pub bgp: u8,
    // Object palettes
    pub obp0: u8,
    pub obp1: u8,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            vram: vec![0; 0x2000].into_boxed_slice(),
            oam: vec![0; 0xA0].into_boxed_slice(),

            // fb: vec![0xFF; FB_SIZE], // initialize to white screen
            mode: Mode::Oam,
            modeclock: 0,
            line: 0,
            lyc: 0,

            coincidence_int: false,
            mode2oam_int: false,
            mode1vblank_int: false,
            mode0hblank_int: false,

            switchbg: true,  // bit 0
            bgmap: false,    // bit 3
            bgtile: true,    // bit 4
            switchlcd: true, // bit 7

            scy: 0,
            scx: 0,

            wy: 0,
            wx: 0,

            // perhaps store these as single bytes, then convert to RGBA when needed
            bgp:  0xFC,
            obp0: 0x00,
            obp1: 0x00,
        }
    }

    pub fn step(&mut self, last_t: usize) {
        self.modeclock += last_t;
        match self.mode {
            Mode::Oam => {
                if self.modeclock >= 80 {
                    self.modeclock = 0;
                    self.mode = Mode::Vram;
                }
            }
            Mode::Vram => {
                if self.modeclock >= 172 {
                    self.modeclock = 0;
                    self.mode = Mode::Hblank;
                    self.renderscan();
                }
            }
            Mode::Hblank => {
                if self.modeclock >= 204 {
                    self.modeclock = 0;
                    self.line += 1;
                    if self.line == 143 {
                        self.mode = Mode::Vblank;
                        // update framebuffer
                    } else {
                        self.mode = Mode::Oam;
                    }
                }
            }
            Mode::Vblank => {
                if self.modeclock >= 456 {
                    self.modeclock = 0;
                    self.line += 1;
                    if self.line > 153 {
                        self.mode = Mode::Oam;
                        self.line = 0;
                    }
                }
            }
        }
    }

    // pub fn read_byte(&self, addr: u16) -> u8 {
    //     match addr {
    //         0xFF40 => self.read_lcd_ctrl(), // LCD Control Reg
    //         0xFF41 => self.read_lcd_stat(),
    //         0xFF42 => self.scy,
    //         0xFF43 => self.scx,
    //         0xFF44 => self.line, // current scanline
    //         0xFF45 => self.lyc,
    //         0xFF47 => panic!("Background Palette is Write-Only!"), // TODO
    //         0xFF48 => panic!("Object Palette 0 is Write-Only!"), // TODO
    //         0xFF49 => panic!("Object Palette 1 is Write-Only!"), // TODO
    //         0xFF4A => self.wy,
    //         0xFF4B => self.wx,
    //         _ => panic!("Reading from PPU IOReg {:#X} not yet implemented!", addr)
    //     }
    // }

    // pub fn write_byte(&mut self, addr: u16, value: u8) {
    //     match addr {
    //         0xFF40 => self.write_lcd_ctrl(value),
    //         0xFF41 => self.write_lcd_stat(value),
    //         0xFF42 => self.scy = value,
    //         0xFF43 => self.scx = value,
    //         0xFF44 => println!("Can't write to 0xFF44."),
    //         0xFF45 => self.lyc = value,
    //         0xFF47 => self.bgp = value, // BG Palette
    //         0xFF48 => self.obp0 = value, // Object Palette 0
    //         0xFF49 => self.obp1 = value, // Object Palette 0
    //         0xFF4A => self.wy = value,
    //         0xFF4B => self.wx = value,
    //         0xFF7F => println!("oopsy! 0xFF7F"),
    //         _ => panic!("Writing to PPU IOReg {:#X} not yet implemented!", addr)
    //     }
    // }

    pub fn read_vram(&self, addr: usize) -> u8 {
        self.vram[addr]
    }

    pub fn read_vram16(&self, addr: usize) -> u16 {
        LittleEndian::read_u16(&self.vram[addr..])
    }

    pub fn write_vram(&mut self, addr: usize, value: u8) {
        self.vram[addr] = value;
    }

    pub fn write_vram16(&mut self, addr: usize, value: u16) {
        LittleEndian::write_u16(&mut self.vram[addr..], value);
    }

    pub fn read_oam(&self, addr: usize) -> u8 {
        self.oam[addr]
    }

    pub fn read_oam16(&self, addr: usize) -> u16 {
        LittleEndian::read_u16(&self.oam[addr..])
    }

    pub fn write_oam(&mut self, addr: usize, value: u8) {
        self.oam[addr] = value;
    }

    pub fn write_oam16(&mut self, addr: usize, value: u16) {
        LittleEndian::write_u16(&mut self.oam[addr..], value);
    }

    pub fn read_lcd_ctrl(&self) -> u8 {
        let bit0 = if self.switchbg  { 0x01 } else { 0x00 };
        let bit3 = if self.bgmap     { 0x08 } else { 0x00 };
        let bit4 = if self.bgtile    { 0x10 } else { 0x00 };
        let bit7 = if self.switchlcd { 0x80 } else { 0x00 };
        bit0 | bit3 | bit4 | bit7
    }

    pub fn write_lcd_ctrl(&mut self, value: u8) {
        self.switchbg  = value & (1 << 0) != 0;
        self.bgmap     = value & (1 << 3) != 0;
        self.bgtile    = value & (1 << 4) != 0;
        self.switchlcd = value & (1 << 4) != 0;
    }

    pub fn read_lcd_stat(&self) -> u8 {
        (if self.coincidence_int  { 1 << 6 } else { 0 }) |
        (if self.mode2oam_int     { 1 << 5 } else { 0 }) |
        (if self.mode1vblank_int  { 1 << 4 } else { 0 }) |
        (if self.mode0hblank_int  { 1 << 3 } else { 0 }) |
        (if self.line == self.lyc { 1 << 2 } else { 0 }) |
        match self.mode {
            Mode::Oam    => 0b10,
            Mode::Vram   => 0b11,
            Mode::Hblank => 0b00,
            Mode::Vblank => 0b01
        }
    }

    pub fn write_lcd_stat(&mut self, value: u8) {
        self.coincidence_int = value & (1 << 6) != 0;
        self.mode2oam_int    = value & (1 << 5) != 0;
        self.mode1vblank_int = value & (1 << 4) != 0;
        self.mode0hblank_int = value & (1 << 3) != 0;
    }

    fn renderscan(&mut self) {
        // VRAM offset for map
        let mut map_offset: usize = if self.bgmap { 0x1C00 } else { 0x1800 };
        // which line of tiles to use in the map
        map_offset += (((self.line as usize) + (self.scy as usize)) & 0xFF) >> 3;
        // which tile to start with in the map line
        let mut line_offset: usize = (self.scx as usize) >> 3;
        // which line of pixels to use in the tiles
        let y = (self.line + self.scy) & 0b111;
        // where in the tile line to start
        let mut x = self.scx & 0b111;
        // where to render in the buffer
        let mut fb_offset = (self.line as usize) * 160;
        // read tile index from bgmap
        // let color;
        let mut tile = self.vram[map_offset + line_offset] as usize;
        // if tile data set in use is #1,
        // indices are signed; calculate real offset
        if self.bgtile && tile < 128 { tile += 256 }

        for i in 0..160 {
            // remap the tile pixel through the palette
            //color = self.pal[self.tileset[tile][y][x]];

            // plot the pixel to the buffer
            // self.fb[fb_offset] = 0;//color;
            fb_offset += 1;

            // when this tile ends, read another
            x += 1;
            if x == 8 {
                x = 0;
                line_offset = (line_offset + 1) & 0x1F;
                tile = self.vram[map_offset + line_offset] as usize;
                if self.bgtile && tile < 128 { tile += 256 };
            }
        }
    }
}

// fn set_palette(pal: &mut Vec<Vec<u8>>, value: u8) {
//     for i in 0..4 {
//         pal[i] = match (value >> (i * 2)) & 0b11 {
//             0 => WHITE.to_vec(),
//             1 => LGRAY.to_vec(),
//             2 => DGRAY.to_vec(),
//             3 => BLACK.to_vec(),
//             _ => unreachable!()
//         }
//     }
// }

#[derive(Debug)]
enum Mode {
    Oam,    // 2
    Vram,   // 3
    Hblank, // 0
    Vblank  // 1
}
