use std::io::{self,Write};

use byteorder::{LittleEndian, ByteOrder};

use dmg::{Cart, Ppu, Apu, Timer}; // TODO more periphs?
use dmg::mem_map::{self, Addr};
use Color;

const RAM_SIZE: usize = 0x2000;

pub struct Interconnect {
    ppu: Ppu,
    apu: Apu,
    timer: Timer,

    in_bootrom: bool,
    boot: Box<[u8]>,
    cart: Cart,

    cgb_ram_bank: u8,
    ram: Box<[u8]>,
    hram: Box<[u8]>,

    // io_regs: Box<[u8]>, // TODO separate into other modules
    serial_byte: u8,
    serial_transfer_start: SerialTransfer,
    serial_clock: SerialClock,
    serial_shift_clock: SerialShift,

    iflags: u8, // TODO break up the bits for store
    dma_addr: u8,
    dma_buffer: u8,
    dma_counter: u8,

    ie_reg: u8 // Interrupts Enable Register TODO break up bits
}

impl Interconnect {
    pub fn new(boot_rom: Box<[u8]>, cart_rom: Box<[u8]>) -> Interconnect {
        Interconnect {
            ppu: Ppu::new(),
            apu: Apu::new(),
            timer: Timer::new(),

            in_bootrom: false,
            boot: boot_rom,
            cart: Cart::new(cart_rom),

            cgb_ram_bank: 0,
            ram: vec![0; RAM_SIZE].into_boxed_slice(),
            hram: vec![0; 128].into_boxed_slice(),

            // io_regs: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //0xFF00
            //               0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            //               0x80, 0xBF, 0xF3, 0x00, 0xBF, 0x00, 0x3F, 0x00, //0xFF10
            //               0x00, 0xBF, 0x7F, 0xFF, 0x9F, 0x00, 0xBF, 0x00,
            //               0xFF, 0x00, 0x00, 0xBF, 0x77, 0xF3, 0xF1, 0x00] //0xFF20
            //     .into_boxed_slice(),
            serial_byte: 0,
            serial_transfer_start: SerialTransfer::No,
            serial_clock: SerialClock::Normal,
            serial_shift_clock: SerialShift::External,

            iflags: 0,
            dma_addr: 0,
            dma_buffer: 0,
            dma_counter: 0xA0,

            ie_reg: 0x00,
        }
    }

    pub fn framebuffer(&self) -> &[Color] {
        self.ppu.framebuffer()
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match mem_map::map_addr(addr) {
            Addr::Rom(offset) => if self.in_bootrom && offset < 0x100 {
                self.boot[offset]
            } else {
                self.cart.rom_read_byte(offset)
            },
            Addr::Vram(offset) => self.ppu.read_vram(offset),
            Addr::Xram(offset) => self.cart.ram_read_byte(offset),
            Addr::Ram(offset) => self.ram[offset],
            Addr::Echo(offset) => self.ram[offset],
            Addr::Oam(offset) => self.ppu.read_oam(offset),
            Addr::Unused => 0xFF,
            Addr::Hram(offset) => self.hram[offset],

            Addr::JoypadReg => 0xFF, // TODO Joypad input
            Addr::SerialData => self.serial_byte, // TODO
            Addr::SerialControl => self.read_serial_control(),
            Addr::TimerDivReg => self.timer.read_div_reg(),
            Addr::TimerCounter => self.timer.read_counter(),
            Addr::TimerModulo => self.timer.modulo,
            Addr::TimerControl => self.timer.read_timer_control(),
            Addr::InterruptFlags => self.iflags, // TODO

            Addr::ApuChan1Sweep => self.apu.read_chan1_sweep(),
            Addr::ApuChan1WaveLength => self.apu.read_chan1_wavelength(),
            Addr::ApuChan1Envelope => self.apu.read_chan1_envelope(),
            Addr::ApuChan1FreqLo => panic!("0xFF13 is write-only!"),
            Addr::ApuChan1FreqHi => self.apu.read_chan1_freq_hi(),

            Addr::ApuChan2WaveLength => self.apu.read_chan2_wavelength(),
            Addr::ApuChan2Envelope => self.apu.read_chan2_envelope(),
            Addr::ApuChan2FreqLo => panic!("0xFF18 is write-only!"),
            Addr::ApuChan2FreqHi => self.apu.read_chan2_freq_hi(),

            Addr::ApuChan3Enable => self.apu.read_chan3_enable(),
            Addr::ApuChan3Length => self.apu.read_chan3_length(),
            Addr::ApuChan3Volume => self.apu.read_chan3_volume(),
            Addr::ApuChan3FreqLo => panic!("0xFF1D is write-only!"),
            Addr::ApuChan3FreqHi => self.apu.read_chan3_freq_hi(),
            Addr::ApuWaveRam(offset) => self.apu.read_wave_pattern_ram(offset),

            Addr::ApuChan4Length => self.apu.read_chan4_length(),
            Addr::ApuChan4Envelope => self.apu.read_chan4_envelope(),
            Addr::ApuChan4PolyCounter => self.apu.read_chan4_polycounter(),
            Addr::ApuChan4CounterConsec => self.apu.read_chan4_counter_consec(),

            Addr::ApuChanControl => self.apu.read_chan_control(),
            Addr::ApuOutputSelect => self.apu.output_select,
            Addr::ApuSoundOnReg => self.apu.read_sound_on_reg(),

            Addr::PpuControlReg => self.ppu.read_lcd_ctrl(),
            Addr::PpuStatusReg => self.ppu.read_lcd_stat(),
            Addr::PpuScrollY => self.ppu.scy,
            Addr::PpuScrollX => self.ppu.scx,
            Addr::PpuLcdY => self.ppu.line,
            Addr::PpuLcdYCompare => self.ppu.lyc,
            Addr::PpuOamDma => self.dma_addr,
            Addr::PpuBgPalette => 0xFF,    // TODO write only?
            Addr::PpuObj0Palette => 0xFF, // TODO write only?
            Addr::PpuObj1Palette => 0xFF, // TODO write only?
            Addr::PpuWindowY => self.ppu.wy,
            Addr::PpuWindowX => self.ppu.wx,

            Addr::CgbSpeedSwitch => 0, // TODO CGB
            Addr::PpuDestVramBank => 0, // TODO CGB
            Addr::BootromDisable => if self.in_bootrom { 1 } else { 0 },
            Addr::CgbRamBank => self.cgb_ram_bank & 0x7,
            Addr::InterruptsEnable => self.ie_reg,
            Addr::FF7F => 0xFF,
        }
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        match mem_map::map_addr(addr) {
            Addr::Rom(offset) => if self.in_bootrom && offset < 0x100 {
                LittleEndian::read_u16(&self.boot[offset..])
            } else {
                self.cart.rom_read_word(offset)
            },
            Addr::Vram(offset) => self.ppu.read_vram16(offset),
            Addr::Xram(offset) => self.cart.ram_read_word(offset),
            Addr::Ram(offset) =>
                LittleEndian::read_u16(&self.ram[offset..]),
            Addr::Echo(offset) =>
                LittleEndian::read_u16(&self.ram[offset..]),
            Addr::Oam(offset) => self.ppu.read_oam16(offset),
            Addr::Unused => 0xFFFF,
            Addr::Hram(offset) =>
                LittleEndian::read_u16(&self.hram[offset..]),

            _ => panic!("Reading 16 bits from IO Regs not supported!")
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match mem_map::map_addr(addr) {
            Addr::Rom(offset) => self.cart.mbc_write_byte(offset, value),
            Addr::Vram(offset) => self.ppu.write_vram(offset, value),
            Addr::Xram(offset) => self.cart.ram_write_byte(offset, value),
            Addr::Ram(offset) => self.ram[offset] = value,
            Addr::Echo(offset) => self.ram[offset] = value,
            Addr::Oam(offset) => self.ppu.write_oam(offset, value),
            Addr::Unused => {},
            Addr::Hram(offset) => self.hram[offset] = value,

            Addr::JoypadReg => {}, // TODO Joypad select
            Addr::SerialData => {
                self.serial_byte = value;
                print!("{}", value as char);
                io::stdout().flush().ok().expect("Could not flush stdout");
            }, // TODO
            Addr::SerialControl => self.write_serial_control(value),
            Addr::TimerDivReg => self.timer.write_div_reg(),
            Addr::TimerCounter => self.timer.write_counter(value),
            Addr::TimerModulo => self.timer.modulo = value,
            Addr::TimerControl => self.timer.write_timer_control(value),
            Addr::InterruptFlags => self.iflags = value, //TODO

            Addr::ApuChan1Sweep => self.apu.write_chan1_sweep(value),
            Addr::ApuChan1WaveLength => self.apu.write_chan1_wavelength(value),
            Addr::ApuChan1Envelope => self.apu.write_chan1_envelope(value),
            Addr::ApuChan1FreqLo => self.apu.write_chan1_freq_lo(value),
            Addr::ApuChan1FreqHi => self.apu.write_chan1_freq_hi(value),

            Addr::ApuChan2WaveLength => self.apu.write_chan2_wavelength(value),
            Addr::ApuChan2Envelope => self.apu.write_chan2_envelope(value),
            Addr::ApuChan2FreqLo => self.apu.write_chan2_freq_lo(value),
            Addr::ApuChan2FreqHi => self.apu.write_chan2_freq_hi(value),

            Addr::ApuChan3Enable => self.apu.write_chan3_enable(value),
            Addr::ApuChan3Length => self.apu.write_chan3_length(value),
            Addr::ApuChan3Volume => self.apu.write_chan3_volume(value),
            Addr::ApuChan3FreqLo => self.apu.write_chan3_freq_lo(value),
            Addr::ApuChan3FreqHi => self.apu.write_chan3_freq_hi(value),
            Addr::ApuWaveRam(offset) =>
                self.apu.write_wave_pattern_ram(offset, value),

            Addr::ApuChan4Length => self.apu.write_chan4_length(value),
            Addr::ApuChan4Envelope => self.apu.write_chan4_envelope(value),
            Addr::ApuChan4PolyCounter => self.apu.write_chan4_polycounter(value),
            Addr::ApuChan4CounterConsec =>
                self.apu.write_chan4_counter_consec(value),

            Addr::ApuChanControl => self.apu.write_chan_control(value),
            Addr::ApuOutputSelect => self.apu.output_select = value,
            Addr::ApuSoundOnReg => self.apu.write_sound_on_reg(value),

            Addr::PpuControlReg => self.ppu.write_lcd_ctrl(value),
            Addr::PpuStatusReg => self.ppu.write_lcd_stat(value),
            Addr::PpuScrollY => self.ppu.scy = value,
            Addr::PpuScrollX => self.ppu.scx = value,
            Addr::PpuLcdY => panic!("Can't change current scanline!"),
            Addr::PpuLcdYCompare => self.ppu.lyc = value,
            Addr::PpuOamDma => {
                self.dma_addr = value;
                self.dma_counter = 0;
                self.dma();
            }
            Addr::PpuBgPalette => self.ppu.write_bg_palette(value),
            Addr::PpuObj0Palette => self.ppu.write_obj0_palette(value),
            Addr::PpuObj1Palette => self.ppu.write_obj1_palette(value),
            Addr::PpuWindowY => self.ppu.wy = value,
            Addr::PpuWindowX => self.ppu.wx = value,

            Addr::CgbSpeedSwitch => {}, // TODO CGB
            Addr::PpuDestVramBank => {}, // TODO CGB
            Addr::BootromDisable => self.in_bootrom = false,
            Addr::CgbRamBank => self.cgb_ram_bank = value & 0x7,
            Addr::InterruptsEnable => self.ie_reg = value,
            Addr::FF7F => {},
        }
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        match mem_map::map_addr(addr) {
            Addr::Rom(offset) => panic!("Write word to MBC not supported"),
            Addr::Vram(offset) => self.ppu.write_vram16(offset, value),
            Addr::Xram(offset) => self.cart.ram_write_word(offset, value),
            Addr::Ram(offset) =>
                LittleEndian::write_u16(&mut self.ram[offset..], value),
            Addr::Echo(offset) =>
                LittleEndian::write_u16(&mut self.ram[offset..], value),
            Addr::Oam(offset) => self.ppu.write_oam16(offset, value),
            Addr::Unused => {},
            Addr::Hram(offset) =>
                LittleEndian::write_u16(&mut self.hram[offset..], value),

            _ => panic!("tried writing word to unrecognized address {:#x}", addr)
        }
    }

    pub fn step(&mut self, cycles: usize) {
        // OAM DMA
        if self.dma_counter < 0xA0 {
            self.dma();
        }

        // Timer Interrupt
        if self.timer.step(cycles) {
            self.iflags |= 1 << 2;
        }

        self.ppu.step(cycles);
        // Vblank Interrupt
        if self.ppu.line == 144 && self.ppu.enter_vblank {
            self.iflags |= 1 << 0;
            self.ppu.enter_vblank = false;
        }

        // LCD Stat Interrupts
        let stat = self.ppu.read_lcd_stat();
        let coincidence_int = stat >> 6 & 1 != 0;
        let mode2_int = stat >> 5 & 1 != 0;
        let mode1_int = stat >> 4 & 1 != 0;
        let mode0_int = stat >> 3 & 1 != 0;
        let coincidence = stat >> 2 & 1 != 0;
        let mode = stat & 0b11;
        if coincidence && coincidence_int && self.ppu.coincidence_start {
            self.ppu.coincidence_start = false;
            self.iflags |= 1 << 1;
        }
        match mode {
            0b00 => if mode0_int && self.ppu.enter_mode0 {
                self.ppu.enter_mode0 = false;
                self.iflags |= 1 << 1;
            },
            0b01 => if (mode1_int || mode2_int) && self.ppu.enter_mode1 {
                self.ppu.enter_mode1 = false;
                self.iflags |= 1 << 1;
            },
            0b10 => if mode2_int && self.ppu.enter_mode2 {
                self.ppu.enter_mode2 = false;
                self.iflags |= 1 << 1;
            },
            _ => {}
        }
    }

    fn dma(&mut self) {
        // TODO check this
        // -1: Read(0)
        // 0: Read(1) Write(0)
        // n: Read(n+1) Write(n)
        let addr = (self.dma_addr as u16) << 8;
        let slice = match mem_map::map_addr(addr) {
            Addr::Rom(offset) => &self.cart.rom[offset..],
            Addr::Ram(offset) => &self.ram[offset..],
            Addr::Vram(offset) => panic!("dma from vram not implemented"),
            Addr::Xram(offset) => &self.cart.ram[offset..],
            Addr::Echo(offset) => &self.ram[offset..],
            _ => panic!("Can't DMA from addresses higher than 0xF100")
        };
        let x = self.dma_counter as usize;
        self.ppu.write_oam(x, slice[x]);
        // self.dma_buffer = slice[x];
        self.dma_counter += 1;
    }

    fn read_serial_control(&self) -> u8 {
        let bit7 = match self.serial_transfer_start {
            SerialTransfer::No => 0,
            SerialTransfer::Start => 1 << 7
        };
        let bit1 = match self.serial_clock {
            SerialClock::Normal => 0,
            SerialClock::Fast => 1 << 1
        };
        let bit0 = match self.serial_shift_clock {
            SerialShift::External => 0,
            SerialShift::Internal => 1 << 0
        };
        bit7 | bit1 | bit0
    }

    fn write_serial_control(&mut self, value: u8) {
        self.serial_transfer_start = match value >> 7 {
            0 => SerialTransfer::No,
            _ => SerialTransfer::Start
        };
        self.serial_clock = match value >> 1 & 1 {
            0 => SerialClock::Normal,
            _ => SerialClock::Fast
        };
        self.serial_shift_clock = match value & 1 {
            0 => SerialShift::External,
            _ => SerialShift::Internal
        }
    }
}

#[derive(Debug)]
enum SerialTransfer {
    No,
    Start
}

#[derive(Debug)]
enum SerialClock {
    Normal,
    Fast
}

#[derive(Debug)]
enum SerialShift {
    External,
    Internal
}
