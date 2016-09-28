// const FB_SIZE: usize = 160*144;
// const WHITE: [u8; 4] = [255, 255, 255, 255];
// const LGRAY: [u8; 4] = [192, 192, 192, 255];
// const DGRAY: [u8; 4] = [ 96,  96,  96, 255];
// const BLACK: [u8; 4] = [  0,   0,   0, 255];
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const SCREEN_AREA: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

use std::cmp::Ordering;
use byteorder::{ByteOrder, LittleEndian};
use Color;

pub struct Ppu {
    vram: Box<[u8]>,
    oam: Box<[Sprite]>,
    tileset: Box<[Tile]>,
    tile_map1: Box<[u8]>,
    tile_map2: Box<[u8]>,

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
            vram: Box::new([0; 0x2000]),
            oam: Box::new([Sprite::new(); 40]),
            tileset: Box::new([Tile::new(); 384]),
            tile_map1: Box::new([0; 0x400]),
            tile_map2: Box::new([0; 0x400]),

            fb: Box::new([Color::Off; SCREEN_AREA]),
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

    pub fn framebuffer(&self) -> &[Color] {
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
                if self.modeclock == 4 {
                    self.draw_line();
                }
                if self.modeclock >= 172 {
                    self.modeclock = 0;
                    self.enter_mode0 = true;
                    self.mode = Mode::Hblank;
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
        match self.mode {
            Mode::Vram => 0xFF,
            _ => match addr {
                0x0000 ... 0x17FF => {
                    let tile = &self.tileset[addr / 16];
                    tile.data[addr % 16]
                },
                0x1800 ... 0x1BFF => self.tile_map1[addr - 0x1800],
                _ => self.tile_map2[addr - 0x1C00]
            }
        }
    }

    pub fn read_vram16(&self, addr: usize) -> u16 {
        match self.mode {
            Mode::Hblank |
            Mode::Vblank |
            Mode::Oam => LittleEndian::read_u16(&self.vram[addr..]),
            _ => 0xFFFF
        }
    }

    pub fn write_vram(&mut self, addr: usize, value: u8) {
        match self.mode {
            Mode::Vram => return,
            _ => match addr {
                0x0000 ... 0x17FF => {
                    let tile = &mut self.tileset[addr / 16];
                    tile.data[addr % 16] = value;
                },
                0x1800 ... 0x1BFF => self.tile_map1[addr - 0x1800] = value,
                _ => self.tile_map2[addr - 0x1C00] = value
            }
        }
    }

    pub fn write_vram16(&mut self, addr: usize, value: u16) {
        match self.mode {
            Mode::Vram => return,
            _ => LittleEndian::write_u16(&mut self.vram[addr..], value)
        }
    }

    pub fn read_oam(&self, addr: usize) -> u8 {
        match self.mode {
            Mode::Hblank |
            Mode::Vblank => {
                let sprite_addr = addr / 4;
                match addr % 4 {
                    0 => self.oam[sprite_addr].y,
                    1 => self.oam[sprite_addr].x,
                    2 => self.oam[sprite_addr].tile,
                    3 => {
                        let sprite = self.oam[sprite_addr];
                        let bit7 = if sprite.bg_prio { 1 << 7 } else { 0 };
                        let bit6 = if sprite.x_flip { 1 << 6 } else { 0 };
                        let bit5 = if sprite.y_flip { 1 << 5 } else { 0 };
                        let bit4 = if sprite.palette { 1 << 4 } else { 0 };
                        bit7 | bit6 | bit5 | bit4
                    },
                    _ => unreachable!()
                }
            },
            _ => 0xFF
        }
    }

    pub fn read_oam16(&self, addr: usize) -> u16 {
        (self.read_oam(addr) as u16) << 8 | self.read_oam(addr + 1) as u16
    }

    pub fn write_oam(&mut self, addr: usize, value: u8) {
        if self.mode == Mode::Vram || self.mode == Mode::Oam {
            return;
        }
        let sprite_addr = addr / 4;
        match addr % 4 {
            0 => self.oam[sprite_addr].y = value,
            1 => self.oam[sprite_addr].x = value,
            2 => self.oam[sprite_addr].tile = value,
            3 => {
                let sprite = &mut self.oam[sprite_addr];
                sprite.bg_prio = value >> 7 != 0;
                sprite.y_flip = value >> 6 & 1 != 0;
                sprite.x_flip = value >> 5 & 1 != 0;
                sprite.palette = value >> 4 & 1 != 0;
            },
            _ => unreachable!()
        }
    }

    pub fn write_oam16(&mut self, addr: usize, value: u16) {
        self.write_oam(addr, (value >> 8) as u8);
        self.write_oam(addr + 1, (value as u8) & 0xFF);
    }

    pub fn dma_from_vram(&mut self, offset: usize) {
        for x in 0x00 .. 0xA0 {
            let byte = self.read_vram(offset + x);
            self.write_oam(x, byte);
        }
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

        if self.bg_display {
            // let map_offset: usize = match self.bg_tilemap_select {
            //     Tilemap::Map0 => 0x1800,
            //     Tilemap::Map1 => 0x1C00
            // };
            let tile_map = if self.bg_tilemap_select == Tilemap::Map1 {
                &self.tile_map2
            } else {
                &self.tile_map1
            };

            let y = self.line.wrapping_add(self.scy);
            let row = (y >> 3) as usize;
            for i in 0..SCREEN_WIDTH {
                let x = (i as u8).wrapping_add(self.scx);
                let col = (x >> 3) as usize;
                let raw_tile_num = tile_map[row * 32 + col];

                let tile_num =
                    if raw_tile_num < 128
                    && self.bg_win_tileset_select == Tileset::Set0 {
                        256 + (raw_tile_num as usize)
                    } else {
                        raw_tile_num as usize
                    };
                let tile = &self.tileset[tile_num];

                let line = (y % 8) * 2;
                let data1 = tile.data[line as usize];
                let data2 = tile.data[line as usize + 1];

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
            // let map_offset: usize = match self.win_tilemap_select {
            //     Tilemap::Map0 => 0x1800,
            //     Tilemap::Map1 => 0x1C00
            // };
            let tile_map = if self.win_tilemap_select == Tilemap::Map1 {
                &self.tile_map2
            } else {
                &self.tile_map1
            };

            let window_x = self.wx.wrapping_sub(7);
            let y = self.line - self.wy;
            let row = (y / 8) as usize;
            for i in (window_x as usize)..SCREEN_WIDTH {
                let mut x = (i as u8).wrapping_add(self.scx);
                if x >= window_x {
                    x = i as u8 - window_x;
                }
                let col = (x / 8) as usize;
                let raw_tile_num = tile_map[row * 32 + col];

                let tile_num =
                    if raw_tile_num < 128
                    && self.bg_win_tileset_select == Tileset::Set0 {
                        256 + raw_tile_num as usize
                    } else {
                        raw_tile_num as usize
                    };
                let tile = &self.tileset[tile_num];

                let line = (y % 8) * 2;
                let data1 = tile.data[line as usize];
                let data2 = tile.data[line as usize + 1];

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

            let mut sprites_to_draw: Vec<(usize, &Sprite)> = self.oam.iter()
                .filter(|sprite| current_line.wrapping_sub(sprite.y) < size)
                .take(10)
                .enumerate()
                .collect();

            sprites_to_draw.sort_by(|&(a_index, a), &(b_index, b)| {
                match a.x.cmp(&b.x) {
                    // if X coords are same, use oam index as priority
                    Ordering::Equal => a_index.cmp(&b_index).reverse(),
                    // use X coord as priority
                    other => other.reverse()
                }
            });

            for (_, sprite) in sprites_to_draw {

                let mut tile_num = sprite.tile as usize;

                let palette = if sprite.palette {
                    &self.obp1
                } else {
                    &self.obp0
                };
                let mut line = if sprite.y_flip {
                    size - current_line.wrapping_sub(sprite.y) - 1
                } else {
                    current_line.wrapping_sub(sprite.y)
                };
                if line >= 8 {
                    tile_num += 1;
                    line -= 8;
                }
                line = line.wrapping_mul(2);
                let tile = &self.tileset[tile_num];
                let data1 = tile.data[line as usize];
                let data2 = tile.data[line as usize + 1];

                for x in (0..8).rev() {
                    let bit =
                        if sprite.x_flip {
                            7 - x
                        } else {
                            x
                        } as usize;
                    let color_value = (((data2 >> bit) & 1) << 1)
                        | ((data1 >> bit) & 1);
                    let raw_color = Color::from_u8(color_value);
                    let color = palette.get(&raw_color);
                    let target_x = sprite.x.wrapping_add(7 - x);
                    if target_x < SCREEN_WIDTH as u8
                        && raw_color != Color::Off
                    {
                        if !sprite.bg_prio || !bg_priority[target_x as usize] {
                            pixels[target_x as usize] = color;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug,PartialEq)]
enum Mode {
    Oam,    // 2
    Vram,   // 3
    Hblank, // 0
    Vblank  // 1
}

#[derive(Debug,PartialEq)]
enum Tileset {
    Set0, // 0x8000-0x8FFF
    Set1, // 0x8800-0x97FF
}

#[derive(Debug,PartialEq)]
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

#[derive(Debug, Clone, Copy)]
struct Sprite {
    y: u8,
    x: u8,
    tile: u8,
    bg_prio: bool,
    y_flip: bool,
    x_flip: bool,
    palette: bool,
    //vram_bank: bool,// CGB only
    //cgb_palette: u8 // 3 bits
}

impl Sprite {
    fn new() -> Sprite {
        Sprite {
            y: 0,
            x: 0,
            tile: 0,
            bg_prio: false,
            y_flip: false,
            x_flip: false,
            palette: false,
        }
    }
}

#[derive(Clone, Copy)]
struct Tile {
    data: [u8; 16]
}

impl Tile {
    fn new() -> Tile {
        Tile {
            data: [0; 16]
        }
    }
}
