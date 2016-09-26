// const FB_SIZE: usize = 160*144;
// const WHITE: [u8; 4] = [255, 255, 255, 255];
// const LGRAY: [u8; 4] = [192, 192, 192, 255];
// const DGRAY: [u8; 4] = [ 96,  96,  96, 255];
// const BLACK: [u8; 4] = [  0,   0,   0, 255];
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const SCREEN_AREA: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

use byteorder::{ByteOrder, LittleEndian};
use Color;

#[derive(Debug)]
pub struct Ppu {
    pub vram: Box<[u8]>,
    pub oam: Box<[u8]>,

    fb: Box<[Color]>,
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
    bgp: Palette,
    // Object palettes
    obp0: Palette,
    obp1: Palette,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            vram: vec![0; 0x2000].into_boxed_slice(),
            oam: vec![0; 0xA0].into_boxed_slice(),

            fb: vec![Color::Off; SCREEN_AREA].into_boxed_slice(),
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
            bgp: Palette::new(),
            obp0: Palette::new(),
            obp1: Palette::new(),
        }
    }

    pub fn framebuffer(&self) -> &Box<[Color]> {
        &self.fb
    }

    pub fn step(&mut self, last_t: usize) {
        if !self.lcd_enable {
            return;
        }
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
                    self.draw_line();
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

    pub fn write_bg_palette(&mut self, value: u8) {
        self.bgp.set(value);
    }

    pub fn write_obj0_palette(&mut self, value: u8) {
        self.obp0.set(value);
    }

    pub fn write_obj1_palette(&mut self, value: u8) {
        self.obp1.set(value);
    }

    fn draw_line(&mut self) {
        let slice_start = (self.line as usize) * SCREEN_WIDTH;
        let slice_end = slice_start + SCREEN_WIDTH;
        let pixels = &mut self.fb[slice_start .. slice_end];
        let mut bg_priority = [false; SCREEN_WIDTH];

        let map_offset: usize = match self.bg_tilemap_select {
            Tilemap::Map0 => 0x1800,
            Tilemap::Map1 => 0x1C00
        };

        if self.bg_display {
            let y = self.line.wrapping_add(self.scy);
            let row = (y >> 3) as usize;
            for i in 0..SCREEN_WIDTH {
                let x = (i as u8).wrapping_add(self.scx);
                let col = (x >> 3) as usize;
                let raw_tile_num = self.vram[map_offset + (row * 32 + col)];

                let tile_num =
                    if let Tileset::Set1 = self.bg_win_tileset_select {
                        raw_tile_num as usize
                    } else if raw_tile_num < 128 {
                        128 + ((raw_tile_num as i8 as i16) + 128) as usize
                    } else {
                        raw_tile_num as usize
                    };

                let line = (y % 8) * 2;
                let data1 = self.vram[tile_num + line as usize];
                let data2 = self.vram[tile_num + line as usize + 1];

                let bit = (x % 8).wrapping_sub(7).wrapping_mul(0xFF) as usize;
                let color_value = ((data2 >> bit) << 1) & 2
                    | ((data1 >> bit) & 1);
                let raw_color = Color::from_u8(color_value);
                let color = self.bgp.get(&raw_color);
                bg_priority[i] = raw_color != Color::Off;
                pixels[i] = color;
            }
        }
        if self.win_display && self.wy <= self.line {
            let window_x = self.wx.wrapping_sub(7);
            let y = self.line - self.wy;
            let row = (y / 8) as usize;
            for i in (window_x as usize)..SCREEN_WIDTH {
                let mut x = (i as u8).wrapping_add(self.scx);
                if x >= window_x {
                    x = i as u8 - window_x;
                }
                let col = (x / 8) as usize;
                let raw_tile_num = self.vram[map_offset + (row * 32 + col)];

                let tile_num =
                    if let Tileset::Set1 = self.bg_win_tileset_select {
                        raw_tile_num as usize
                    } else if raw_tile_num < 128 {
                        128 + ((raw_tile_num as i8 as i16) + 128) as usize
                    } else {
                        raw_tile_num as usize
                    };

                let line = (y % 8) * 2;
                let data1 = self.vram[tile_num + line as usize];
                let data2 = self.vram[tile_num + line as usize + 1];

                let bit = (x % 8).wrapping_sub(7).wrapping_mul(0xFF) as usize;
                let color_value = ((data2 >> bit) << 1) & 2
                    | ((data1 >> bit) & 1);
                let raw_color = Color::from_u8(color_value);
                let color = self.bgp.get(&raw_color);
                bg_priority[i] = raw_color != Color::Off;
                pixels[i] = color;
            }
        }
        if self.obj_display {
            let size = match self.obj_size {
                SpriteSize::Normal => 8,
                SpriteSize::DblHeight => 16
            };

            let current_line = self.line;
            for i in 0..0x28 {
                let offset = i * 4;
                let sprite_y = self.oam[offset];
                let sprite_x = self.oam[offset + 1];
                let mut tile_num = self.oam[offset + 2] as usize;
                let flags = self.oam[offset + 3];

                let palette = match flags >> 4 & 1 {
                    0 => &self.obp0,
                    _ => &self.obp1
                };
                let mut line = if flags >> 6 & 1 != 0 {
                    size - current_line.wrapping_sub(sprite_y) - 1
                } else {
                    current_line.wrapping_sub(sprite_y)
                };
                if line >= 8 {
                    tile_num += 1;
                    line -= 8;
                }
                line *= 2;
                // let tile = self.vram[tile_num];
                let data1 = self.vram[tile_num + line as usize];
                let data2 = self.vram[tile_num + line as usize + 1];

                for x in (0..8).rev() {
                    let bit =
                        if flags >> 5 & 1 != 0 {
                            7 - x
                        } else {
                            x
                        } as usize;
                    let color_value = ((data2 >> bit) << 1) & 2
                        | ((data1 >> bit) & 1);
                    let raw_color = Color::from_u8(color_value);
                    let color = palette.get(&raw_color);
                    let target_x = sprite_x.wrapping_add(7 - x);
                    if target_x < SCREEN_WIDTH as u8
                        && raw_color != Color::Off
                    {
                        if flags >> 7 == 0 || !bg_priority[target_x as usize] {
                            pixels[target_x as usize] = color;
                        }
                    }
                }
            }
        }
    }
}

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

#[derive(Debug)]
struct Palette {
    off: Color,
    light: Color,
    dark: Color,
    on: Color
}

impl Palette {
    fn new() -> Palette {
        Palette {
            off: Color::On,
            light: Color::On,
            dark: Color::On,
            on: Color::On,
        }
    }

    fn get(&self, color: &Color) -> Color {
        match *color {
            Color::Off => self.off,
            Color::Light => self.light,
            Color::Dark => self.dark,
            Color::On => self.on
        }
    }

    fn set(&mut self, value: u8) {
        self.off = Color::from_u8((value >> 0) & 0b11);
        self.light = Color::from_u8((value >> 2) & 0b11);
        self.dark = Color::from_u8((value >> 4) & 0b11);
        self.on = Color::from_u8((value >> 6) & 0b11);
    }
}
