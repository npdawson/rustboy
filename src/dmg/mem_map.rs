const RAM_SIZE: u16 = 0x2000;
const ROM_BANK_SIZE: u16 = 0x4000;

const ROM_START: u16 = 0x0000;
const ROM_END: u16 = 2 * ROM_BANK_SIZE - 1;

const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = VRAM_START + RAM_SIZE - 1;

const XRAM_START: u16 = 0xA000;
const XRAM_END: u16 = XRAM_START + RAM_SIZE - 1;

const WRAM_START: u16 = 0xC000;
const WRAM_END: u16 = WRAM_START + RAM_SIZE - 1;

const ECHO_START: u16 = 0xE000;
const ECHO_END: u16 = 0xFDFF;

const OAM_START: u16 = 0xFE00;
const OAM_SIZE: u16 = 0xA0;
const OAM_END: u16 = OAM_START + OAM_SIZE - 1;

const UNUSED_START: u16 = 0xFEA0;
const UNUSED_END: u16 = 0xFEFF;

const JOYPAD_REG: u16 = 0xFF00;
const SERIAL_DATA: u16 = 0xFF01;
const SERIAL_CTRL: u16 = 0xFF02;
const TIMER_DIV_REG: u16 = 0xFF04;
const TIMER_COUNTER: u16 = 0xFF05;
const TIMER_MODULO: u16 = 0xFF06;
const TIMER_CTRL: u16 = 0xFF07;
const IFLAGS: u16 = 0xFF0F;

const APU_CHAN1_SWEEP: u16 = 0xFF10;
const APU_CHAN1_WAVELENGTH: u16 = 0xFF11;
const APU_CHAN1_ENVELOPE: u16 = 0xFF12;
const APU_CHAN1_FREQ_LO: u16 = 0xFF13;
const APU_CHAN1_FREQ_HI: u16 = 0xFF14;

const APU_CHAN2_WAVELENGTH: u16 = 0xFF16;
const APU_CHAN2_ENVELOPE: u16 = 0xFF17;
const APU_CHAN2_FREQ_LO: u16 = 0xff18;
const APU_CHAN2_FREQ_HI: u16 = 0xFF19;

const APU_CHAN3_ENABLE: u16 = 0xFF1A;
const APU_CHAN3_LENGTH: u16 = 0xff1b;
const APU_CHAN3_VOLUME: u16 = 0xFF1C;
const APU_CHAN3_FREQ_LO: u16 = 0xff1d;
const APU_CHAN3_FREQ_HI: u16 = 0xff1e;
const APU_WAVE_RAM_START: u16 = 0xFF30;
const APU_WAVE_RAM_LENGTH: u16 = 0x0010;
const APU_WAVE_RAM_END: u16 = APU_WAVE_RAM_START + APU_WAVE_RAM_LENGTH - 1;

const APU_CHAN4_LENGTH: u16 = 0xff20;
const APU_CHAN4_ENVELOPE: u16 = 0xFF21;
const APU_CHAN4_POLYCOUNTER: u16 = 0xff22;
const APU_CHAN4_COUNTER_CONSEC: u16 = 0xFF23;

const APU_CHAN_CONTROL: u16 = 0xFF24;
const APU_OUTPUT_SELECT: u16 = 0xFF25;
const APU_SOUND_ON_REG: u16 = 0xFF26;

const PPU_CONTROL_REG: u16 = 0xFF40;
const PPU_STATUS_REG: u16 = 0xFF41;
const PPU_SCROLL_Y: u16 = 0xFF42;
const PPU_SCROLL_X: u16 = 0xFF43;
const PPU_LCD_Y: u16 = 0xFF44;
const PPU_LCD_Y_COMPARE: u16 = 0xFF45;
const PPU_OAM_DMA: u16 = 0xFF46;
const PPU_BG_PALETTE: u16 = 0xFF47;
const PPU_OBJ0_PALETTE: u16 = 0xFF48;
const PPU_OBJ1_PALETTE: u16 = 0xFF49;
const PPU_WINDOW_Y: u16 = 0xFF4A;
const PPU_WINDOW_X: u16 = 0xFF4B;

const CGB_SPEED_SWITCH: u16 = 0xFF4D;
const CGB_VRAM_BANK: u16 = 0xFF4F;
const BOOTROM_DISABLE: u16 = 0xFF50;
const CGB_IR_COMMS: u16 = 0xff56;
const CGB_RAM_BANK: u16 = 0xFF70;

const HRAM_START: u16 = 0xFF80;
const HRAM_SIZE: u16 = 0x007F;
const HRAM_END: u16 = HRAM_START + HRAM_SIZE - 1;

const IEREG: u16 = 0xFFFF;

#[derive(Debug)]
pub enum Addr {
    Rom(usize),
    Vram(usize),
    Xram(usize),
    Ram(usize),
    Echo(usize),
    Oam(usize),
    Unused,
    Hram(usize),

    JoypadReg,      // FF00 P1 Joypad Input
    SerialData,     // FF01 SB
    SerialControl,  // FF02 SC

    TimerDivReg,    // FF04 DIV
    TimerCounter,   // FF05 TIMA
    TimerModulo,    // FF06 TMA
    TimerControl,   // FF07 TAC

    InterruptFlags, // FF0F IF

// Name Addr 7654 3210 Function
// -----------------------------------------------------------------
//        Square 1
// NR10 FF10 -PPP NSSS Sweep period, negate, shift
// NR11 FF11 DDLL LLLL Duty, Length load (64-L)
// NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
// NR13 FF13 FFFF FFFF Frequency LSB
// NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB

    ApuChan1Sweep,
    ApuChan1WaveLength, // FF11
    ApuChan1Envelope,   // FF12
    ApuChan1FreqLo,     // FF13
    ApuChan1FreqHi,     // FF14

//        Square 2
//      FF15 ---- ---- Not used
// NR21 FF16 DDLL LLLL Duty, Length load (64-L)
// NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
// NR23 FF18 FFFF FFFF Frequency LSB
// NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB

    ApuChan2WaveLength,
    ApuChan2Envelope,
    ApuChan2FreqLo,
    ApuChan2FreqHi,

//        Wave
// NR30 FF1A E--- ---- DAC power
// NR31 FF1B LLLL LLLL Length load (256-L)
// NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
// NR33 FF1D FFFF FFFF Frequency LSB
// NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB

    ApuChan3Enable,
    ApuChan3Length,
    ApuChan3Volume,
    ApuChan3FreqLo,
    ApuChan3FreqHi,

//        Noise
//      FF1F ---- ---- Not used
// NR41 FF20 --LL LLLL Length load (64-L)
// NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
// NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
// NR44 FF23 TL-- ---- Trigger, Length enable

    ApuChan4Length,
    ApuChan4Envelope,
    ApuChan4PolyCounter,
    ApuChan4CounterConsec,

//        Control/Status
// NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
// NR51 FF25 NW21 NW21 Left enables, Right enables
// NR52 FF26 P--- NW21 Power control/status, Channel length statuses

    ApuChanControl, // FF24
    ApuOutputSelect, // FF25
    ApuSoundOnReg, // FF26

//        Not used
//      FF27 ---- ----
//      .... ---- ----
//      FF2F ---- ----

//        Wave Table
//      FF30 0000 1111 Samples 0 and 1
//      ....
//      FF3F 0000 1111 Samples 30 and 31

    ApuWaveRam(usize),

    PpuControlReg,  // FF40 LCDC
    PpuStatusReg,   // FF41 STAT
    PpuScrollY,     // FF42 SCY
    PpuScrollX,     // FF43 SCX
    PpuLcdY,        // FF44 LY
    PpuLcdYCompare, // FF45 LYC
    PpuOamDma,      // FF46 DMA
    PpuBgPalette,   // FF47 BGP
    PpuObj0Palette, // FF48 OBP0
    PpuObj1Palette, // FF49 OBP1
    PpuWindowY,     // FF4A WY
    PpuWindowX,     // FF4B WX

    CgbSpeedSwitch, // FFAD, cpu speed switch & status
    PpuDestVramBank, // FF4F
    BootromDisable,  // FF50
    // HDMA1         // FF51 (CGB only) New DMA Source, High
    // HDMA2         // FF52 (CGB only) New DMA Source, Low
    // HDMA3         // FF53 (CGB only) New DMA Dest, High
    // HDMA4         // FF54 (CGB only) New DMA Dest, Low
    // HDMA5         // FF55 (CGB only) New DMA Length/Mode/Start
    CgbIrComms,      // FF56 (CGB only) IR Comm Port
    // CGB BCPS/BGPI // FF68 Background Palette Index
    // CGB BCPD/BGPD // FF69 Background Palette Data
    // CGB OCPS/OBPI // FF6A SpritePalette Index
    // CGB OCPD/OBPD // FF6B Sprite Palette Data
    // FF6C - Undocumented (FEh) - Bit 0 (Read/Write) - CGB Mode Only
    CgbRamBank,      // FF70 SVBK
    // FF6C - Undocumented (FEh) - Bit 0 (Read/Write) - CGB Mode Only
    // FF72 - Undocumented (00h) - Bit 0-7 (Read/Write)
    // FF73 - Undocumented (00h) - Bit 0-7 (Read/Write)
    // FF74 - Undocumented (00h) - Bit 0-7 (Read/Write) - CGB Mode Only
    // FF75 - Undocumented (8Fh) - Bit 4-6 (Read/Write)
    // FF76 - Undocumented (00h) - (Read Only)
    // PCM amplitude, channels 1 and 2.
    // FF77 - Undocumented (00h) - (Read Only)
    // PCM amplitude, channels 3 and 4.
    InterruptsEnable, // FFFF
    FF7F
}

pub fn map_addr(addr: u16) -> Addr {
    match addr {
        ROM_START ... ROM_END =>
            Addr::Rom((addr - ROM_START) as usize),
        VRAM_START ... VRAM_END =>
            Addr::Vram((addr - VRAM_START) as usize),
        XRAM_START ... XRAM_END =>
            Addr::Xram((addr - XRAM_START) as usize),
        WRAM_START ... WRAM_END =>
            Addr::Ram((addr - WRAM_START) as usize),
        ECHO_START ... ECHO_END =>
            Addr::Echo((addr - ECHO_START) as usize),
        OAM_START ... OAM_END =>
            Addr::Oam((addr - OAM_START) as usize),
        UNUSED_START ... UNUSED_END => Addr::Unused,
        HRAM_START ... HRAM_END =>
            Addr::Hram((addr - HRAM_START) as usize),

        JOYPAD_REG => Addr::JoypadReg,
        SERIAL_DATA => Addr::SerialData,
        SERIAL_CTRL => Addr::SerialControl,
        TIMER_DIV_REG => Addr::TimerDivReg,
        TIMER_COUNTER => Addr::TimerCounter,
        TIMER_MODULO => Addr::TimerModulo,
        TIMER_CTRL => Addr::TimerControl,
        IFLAGS => Addr::InterruptFlags,

        APU_CHAN1_SWEEP => Addr::ApuChan1Sweep,
        APU_CHAN1_WAVELENGTH => Addr::ApuChan1WaveLength,
        APU_CHAN1_ENVELOPE => Addr::ApuChan1Envelope,
        APU_CHAN1_FREQ_LO => Addr::ApuChan1FreqLo,
        APU_CHAN1_FREQ_HI => Addr::ApuChan1FreqHi,

        APU_CHAN2_WAVELENGTH => Addr::ApuChan2WaveLength,
        APU_CHAN2_ENVELOPE => Addr::ApuChan2Envelope,
        APU_CHAN2_FREQ_LO => Addr::ApuChan2FreqLo,
        APU_CHAN2_FREQ_HI => Addr::ApuChan2FreqHi,

        APU_CHAN3_ENABLE => Addr::ApuChan3Enable,
        APU_CHAN3_LENGTH => Addr::ApuChan3Length,
        APU_CHAN3_VOLUME => Addr::ApuChan3Volume,
        APU_CHAN3_FREQ_LO => Addr::ApuChan3FreqLo,
        APU_CHAN3_FREQ_HI => Addr::ApuChan3FreqHi,
        APU_WAVE_RAM_START ... APU_WAVE_RAM_END =>
            Addr::ApuWaveRam((addr - APU_WAVE_RAM_START) as usize),

        APU_CHAN4_LENGTH => Addr::ApuChan4Length,
        APU_CHAN4_ENVELOPE => Addr::ApuChan4Envelope,
        APU_CHAN4_POLYCOUNTER => Addr::ApuChan4PolyCounter,
        APU_CHAN4_COUNTER_CONSEC => Addr::ApuChan4CounterConsec,

        APU_CHAN_CONTROL => Addr::ApuChanControl,
        APU_OUTPUT_SELECT => Addr::ApuOutputSelect,
        APU_SOUND_ON_REG => Addr::ApuSoundOnReg,

        PPU_CONTROL_REG => Addr::PpuControlReg,
        PPU_STATUS_REG => Addr::PpuStatusReg,
        PPU_SCROLL_Y => Addr::PpuScrollY,
        PPU_SCROLL_X => Addr::PpuScrollX,
        PPU_LCD_Y => Addr::PpuLcdY,
        PPU_LCD_Y_COMPARE => Addr::PpuLcdYCompare,
        PPU_OAM_DMA => Addr::PpuOamDma,
        PPU_BG_PALETTE => Addr::PpuBgPalette,
        PPU_OBJ0_PALETTE => Addr::PpuObj0Palette,
        PPU_OBJ1_PALETTE => Addr::PpuObj1Palette,
        PPU_WINDOW_Y => Addr::PpuWindowY,
        PPU_WINDOW_X => Addr::PpuWindowX,

        CGB_SPEED_SWITCH => Addr::CgbSpeedSwitch,
        CGB_VRAM_BANK => Addr::PpuDestVramBank,
        BOOTROM_DISABLE => Addr::BootromDisable,
        CGB_IR_COMMS => Addr::CgbIrComms,
        CGB_RAM_BANK => Addr::CgbRamBank,
        IEREG => Addr::InterruptsEnable,
        0xFF7F => Addr::FF7F,
        _ => panic!("Unrecognized address: {:#x}", addr)
    }
}
