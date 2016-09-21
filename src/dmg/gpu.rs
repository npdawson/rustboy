const FB_SIZE: usize = 160*144;
const VRAM_START: usize = 0x8000;
const WHITE: [u8; 4] = [255, 255, 255, 255];
const LGRAY: [u8; 4] = [192, 192, 192, 255];
const DGRAY: [u8; 4] = [ 96,  96,  96, 255];
const BLACK: [u8; 4] = [  0,   0,   0, 255];

use minifb::Window;
use byteorder::{ByteOrder, LittleEndian};

pub struct Gpu {
    vram: Vec<u8>,

    fb: Vec<u32>,
    mode: Mode,
    modeclock: usize,
    line: u8, // 160 lines
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
    scy: u8,
    scx: u8,
    // Window coords
    wy: u8,
    wx: u8,
    // background palette
    bgp: Vec<Vec<u8>>,
    // Object palettes
    obp0: Vec<Vec<u8>>,
    obp1: Vec<Vec<u8>>,
}

impl Gpu {
    pub fn new() -> Gpu {
        Gpu {
            vram: vec![0; 0x2000],

            fb: vec![0xFF; FB_SIZE], // initialize to white screen
            mode: Mode::Oam,
            modeclock: 0,
            line: 0,

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
            bgp:  vec![WHITE.to_vec(), WHITE.to_vec(), WHITE.to_vec(), BLACK.to_vec()],
            obp0: vec![WHITE.to_vec(), WHITE.to_vec(), WHITE.to_vec(), WHITE.to_vec()],
            obp1: vec![WHITE.to_vec(), WHITE.to_vec(), WHITE.to_vec(), WHITE.to_vec()],
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

    pub fn read_byte(&self, addr: usize) -> u8 {
        match addr {
            0xFF40 => self.lcd_ctrl(), // LCD Control Reg
            0xFF41 => self.read_lcd_stat(),
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.line, // current scanline
            0xFF47 => panic!("Background Palette is Write-Only!"), // TODO
            0xFF48 => panic!("Object Palette 0 is Write-Only!"), // TODO
            0xFF49 => panic!("Object Palette 1 is Write-Only!"), // TODO
            _ => panic!("Reading from GPU IOReg {:#X} not yet implemented!", addr)
        }
    }

    pub fn write_byte(&mut self, value: u8, addr: usize) {
        match addr {
            0xFF40 => self.set_lcd_ctrl(value),
            0xFF41 => self.write_lcd_stat(value),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => println!("Can't write to 0xFF44."),
            0xFF47 => set_palette(&mut self.bgp, value), // BG Palette
            0xFF48 => set_palette(&mut self.obp0, value), // Object Palette 0
            0xFF49 => set_palette(&mut self.obp1, value), // Object Palette 0
            0xFF7F => println!("oopsy! 0xFF7F"),
            _ => panic!("Writing to GPU IOReg {:#X} not yet implemented!", addr)
        }
    }

    pub fn read_vram(&self, addr: usize) -> u8 {
        self.vram[addr - VRAM_START]
    }

    pub fn read_vram16(&self, addr: usize) -> u16 {
        LittleEndian::read_u16(&self.vram[addr - VRAM_START..])
    }

    pub fn write_vram(&mut self, value: u8, addr: usize) {
        self.vram[addr - VRAM_START] = value;
    }

    pub fn write_vram16(&mut self, value: u16, addr: usize) {
        LittleEndian::write_u16(&mut self.vram[addr - VRAM_START..], value);
    }

    fn set_lcd_ctrl(&mut self, value: u8) {
        self.switchbg  = value & (1 << 0) != 0;
        self.bgmap     = value & (1 << 3) != 0;
        self.bgtile    = value & (1 << 4) != 0;
        self.switchlcd = value & (1 << 4) != 0;
    }

    fn lcd_ctrl(&self) -> u8 {
        let bit0 = if self.switchbg  { 0x01 } else { 0x00 };
        let bit3 = if self.bgmap     { 0x08 } else { 0x00 };
        let bit4 = if self.bgtile    { 0x10 } else { 0x00 };
        let bit7 = if self.switchlcd { 0x80 } else { 0x00 };
        bit0 | bit3 | bit4 | bit7
    }

    fn read_lcd_stat(&self) -> u8 {
        (if self.coincidence_int { 1 << 6 } else { 0 }) |
        (if self.mode2oam_int    { 1 << 5 } else { 0 }) |
        (if self.mode1vblank_int { 1 << 4 } else { 0 }) |
        (if self.mode0hblank_int { 1 << 3 } else { 0 }) |
        (if self.line == 0       { 1 << 2 } else { 0 }) | // TODO LY @ FF44 == LYC @ FF45
        match self.mode {
            Mode::Oam    => 0b10,
            Mode::Vram   => 0b11,
            Mode::Hblank => 0b00,
            Mode::Vblank => 0b01
        }
    }

    fn write_lcd_stat(&mut self, value: u8) {
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
            self.fb[fb_offset] = 0;//color;
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

fn set_palette(pal: &mut Vec<Vec<u8>>, value: u8) {
    for i in 0..4 {
        pal[i] = match (value >> (i * 2)) & 0b11 {
            0 => WHITE.to_vec(),
            1 => LGRAY.to_vec(),
            2 => DGRAY.to_vec(),
            3 => BLACK.to_vec(),
            _ => unreachable!()
        }
    }
}

enum Mode {
    Oam,    // 2
    Vram,   // 3
    Hblank, // 0
    Vblank  // 1
}
