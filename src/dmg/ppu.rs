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
    pub enter_vblank: bool,
    // LY Compare
    pub lyc: u8,
    // LCD CTRL, make separate struct?
    bg_display: bool, // bit 0
    obj_display: bool, // bit 1
    obj_size: SpriteSize, // bit 2
    bg_tilemap_select: Tilemap, // bit 3
    bg_win_tileset_select: Tileset, // bit 4
    win_display: bool, // bit 5
    win_tilemap_select: Tilemap, // bit 6
    lcd_enable: bool, // bit 7
    // LCD STAT, make separate struct?
    coincidence_int: bool,
    pub coincidence_start: bool,
    mode2oam_int: bool,
    pub enter_mode2: bool,
    mode1vblank_int: bool,
    pub enter_mode1: bool,
    mode0hblank_int: bool,
    pub enter_mode0: bool,
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
            enter_vblank: false,
            lyc: 0,

            coincidence_int: false,
            coincidence_start: false,
            mode2oam_int: false,
            enter_mode2: false,
            mode1vblank_int: false,
            enter_mode1: false,
            mode0hblank_int: false,
            enter_mode0: false,

            bg_display: true,  // bit 0
            obj_display: true,
            obj_size: SpriteSize::Normal,
            bg_tilemap_select: Tilemap::Map0,    // bit 3
            bg_win_tileset_select: Tileset::Set0,    // bit 4
            win_display: true,
            win_tilemap_select: Tilemap::Map0,
            lcd_enable: true, // bit 7

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
                    self.enter_mode0 = true;
                    self.mode = Mode::Hblank;
                    self.renderscan();
                }
            }
            Mode::Hblank => {
                if self.modeclock >= 204 {
                    self.modeclock = 0;
                    self.line += 1;
                    if self.line == 144 {
                        self.enter_vblank = true;
                        self.enter_mode1 = true;
                        self.mode = Mode::Vblank;
                        // TODO update framebuffer
                    } else {
                        self.enter_mode2 = true;
                        self.mode = Mode::Oam;
                    }
                }
            }
            Mode::Vblank => {
                if self.modeclock >= 456 {
                    self.modeclock = 0;
                    self.line += 1;
                    if self.line == 153 {
                        self.enter_mode2 = true;
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
        match self.mode {
            Mode::Hblank |
            Mode::Vblank => self.oam[addr],
            _ => 0xFF
        }
    }

    pub fn read_oam16(&self, addr: usize) -> u16 {
        match self.mode {
            Mode::Hblank |
            Mode::Vblank => LittleEndian::read_u16(&self.oam[addr..]),
            _ => 0xFFFF
        }
    }

    pub fn write_oam(&mut self, addr: usize, value: u8) {
        self.oam[addr] = value;
    }

    pub fn write_oam16(&mut self, addr: usize, value: u16) {
        LittleEndian::write_u16(&mut self.oam[addr..], value);
    }

    pub fn read_lcd_ctrl(&self) -> u8 {
        let bit0 = if self.bg_display  { 1 << 0 } else { 0 };
        let bit1 = if self.obj_display { 1 << 1 } else { 0 };
        let bit2 = match self.obj_size {
            SpriteSize::Normal => 1 << 2,
            SpriteSize::DblHeight => 0
        };
        let bit3 = match self.bg_tilemap_select {
            Tilemap::Map1 => 1 << 3,
            Tilemap::Map0 => 0
        };
        let bit4 = match self.bg_win_tileset_select {
            Tileset::Set0 => 1 << 4,
            Tileset::Set1 => 0
        };
        let bit5 = if self.win_display { 1 << 5 } else { 0 };
        let bit6 = match self.win_tilemap_select {
            Tilemap::Map1 => 1 << 6,
            Tilemap::Map0 => 0
        };
        let bit7 = if self.lcd_enable { 1 << 7 } else { 0 };
        bit0 | bit1 | bit2| bit3 | bit4 | bit5 | bit6 | bit7
    }

    pub fn write_lcd_ctrl(&mut self, value: u8) {
        self.bg_display  = value & (1 << 0) != 0;
        self.obj_display = value & (1 << 1) != 0;
        self.obj_size = if value & (1 << 2) != 0 {
            SpriteSize::DblHeight
        } else {
            SpriteSize::Normal
        };
        self.bg_tilemap_select = if value & (1 << 3) != 0 {
            Tilemap::Map1
        } else {
            Tilemap::Map0
        };
        self.bg_win_tileset_select = if value & (1 << 4) != 0 {
            Tileset::Set0
        } else {
            Tileset::Set1
        };
        self.win_display = value & (1 << 5) != 0;
        self.win_tilemap_select = if value & (1 << 6) != 0 {
            Tilemap::Map1
        } else {
            Tilemap::Map0
        };
        self.lcd_enable = value & (1 << 7) != 0;
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
        let mut map_offset: usize = match self.bg_tilemap_select {
            Tilemap::Map0 => 0x1800,
            Tilemap::Map1 => 0x1C00
        };
        // which line of tiles to use in the map
        map_offset += (self.scy.wrapping_add(self.line) >> 3) as usize;
        // which tile to start with in the map line
        let mut line_offset: usize = (self.scx >> 3) as usize;
        // which line of pixels to use in the tiles
        let y = (self.scy.wrapping_add(self.line)) & 0b111;
        // where in the tile line to start
        let mut x = self.scx & 0b111;
        // where to render in the buffer
        let mut fb_offset = (self.line as usize) * 160;
        // read tile index from bg_tilemap_select
        // let color;
        let mut tile = self.vram[map_offset + line_offset] as usize;
        // if tile data set in use is #1,
        // indices are signed; calculate real offset
        if let Tileset::Set0 = self.bg_win_tileset_select {
            if tile < 128 {
                tile += 256;
            }
        }

        for i in 0..160 {
            // remap the tile pixel through the palette
            //color = self.pal[self.tileset[tile][y][x]];

            // plot the pixel to the buffer
            // self.fb[fb_offset] = color;
            fb_offset += 1;

            // when this tile ends, read another
            x += 1;
            if x == 8 {
                x = 0;
                line_offset = (line_offset + 1) & 0x1F;
                tile = self.vram[map_offset + line_offset] as usize;
                if let Tileset::Set0 = self.bg_win_tileset_select {
                    if tile < 128 {
                        tile += 256;
                    }
                }
            }
        }
    }

    fn draw_bg(&mut self) {

    }

    fn draw_win(&mut self) {

    }

    fn draw_sprites(&mut self) {

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

#[derive(Debug)]
enum Tileset {
    Set0, // 0x8000-0x8FFF
    Set1, // 0x8800-0x97FF
}

#[derive(Debug)]
enum Tilemap {
    Map0, // 0x9800-0x9BFF
    Map1  // 0x9C00-0x9FFF
}

#[derive(Debug)]
enum SpriteSize {
    Normal, // 8x8
    DblHeight // 8x16
}
