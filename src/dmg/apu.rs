#[derive(Debug)]
pub struct Apu {
    // FF11 NR11 Channel 1 Sound length/Wave pattern duty
    channel1_wave: WaveDuty,
    channel1_length: u8,

    // FF12 NR12 Channel 1 Volume Envelope
    channel1_envelope_volume: u8,
    channel1_envelope_direction: EnvDir,
    channel1_envelope_sweeps: u8,

    // FF13 NR13 Channel 1 Frequency Lo
    channel1_frequency: u16,
    // FF14 NR14 Channel 1 Frequency Hi
    // bit 7 Initial (1 = restart sound)
    channel1_counter_consecutive: bool, // bit 6 (1 = stop output when length expires)

    // FF24 NR50 Channel Control
    so2_output_enable: bool,
    so2_output_volume: u8,
    so1_output_enable: bool,
    so1_output_volume: u8,

    // FF25 NR51 Output Select
    pub output_select: u8, // TODO break up bits?

    // FF26 NR52
    enable_sound_controller: bool, // bit 7
    sound_4_on: bool,              // bit 3
    sound_3_on: bool,              // bit 2
    sound_2_on: bool,              // bit 1
    sound_1_on: bool,              // bit 0
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            channel1_wave: WaveDuty::Half,
            channel1_length: 0x3F,

            channel1_envelope_volume: 0xF,
            channel1_envelope_direction: EnvDir::Down,
            channel1_envelope_sweeps: 3,

            channel1_frequency: 0x70,
            channel1_counter_consecutive: false,

            so2_output_enable: false,
            so2_output_volume: 7,
            so1_output_enable: false,
            so1_output_volume: 7,

            output_select: 0xF3,

            enable_sound_controller: true,
            sound_4_on: false,
            sound_3_on: false,
            sound_2_on: false,
            sound_1_on: true,
        }
    }

    pub fn read_chan1_wavelength(&self) -> u8 {
        let bit76 = match self.channel1_wave {
            WaveDuty::Eighth  => 0b00 << 6,
            WaveDuty::Quarter => 0b01 << 6,
            WaveDuty::Half    => 0b10 << 6,
            WaveDuty::ThreeQuarters => 0x11 << 6,
        };
        bit76 | self.channel1_length
    }

    pub fn write_chan1_wavelength(&mut self, value: u8) {
        self.channel1_wave = match value >> 6 {
            0b00 => WaveDuty::Eighth,
            0b01 => WaveDuty::Quarter,
            0b10 => WaveDuty::Half,
            0b11 => WaveDuty::ThreeQuarters,
            _ => unreachable!()
        };
        self.channel1_length = value & 0x3F;
    }

    pub fn read_chan1_envelope(&self) -> u8 {
        let direction = match self.channel1_envelope_direction {
            EnvDir::Down => 0,
            EnvDir::Up   => 1
        };
        self.channel1_envelope_volume << 4
            | direction
            | self.channel1_envelope_sweeps
    }

    pub fn write_chan1_envelope(&mut self, value: u8) {
        self.channel1_envelope_volume = value >> 4;
        self.channel1_envelope_direction = match value >> 3 & 1 {
            0 => EnvDir::Down,
            _ => EnvDir::Up
        };
        self.channel1_envelope_sweeps = value & 0b111;
    }

    pub fn write_chan1_freq_lo(&mut self, value: u8) {
        self.channel1_frequency = self.channel1_frequency & 0xF0 | value as u16;
    }

    pub fn read_chan1_freq_hi(&self) -> u8 {
        if self.channel1_counter_consecutive { 1 << 6 } else { 0 }
    }

    pub fn write_chan1_freq_hi(&mut self, value: u8) {
        if value >> 7 != 0 {
            self.sound_1_on = true;
        }
        let freq_hi = ((value as u16) & 0b111) << 8;
        self.channel1_frequency = self.channel1_frequency & 0xF | freq_hi;
    }

    pub fn read_chan_control(&self) -> u8 {
        let bit7 = if self.so2_output_enable { 1 << 7 } else { 0 };
        let bit3 = if self.so1_output_enable { 1 << 3 } else { 0 };
        bit7 | self.so2_output_volume << 4 | bit3 | self.so1_output_volume
    }

    pub fn write_chan_control(&mut self, value: u8) {
        self.so2_output_enable = value >> 7 != 0;
        self.so2_output_volume = value >> 4 & 0b111;
        self.so1_output_enable = value >> 3 & 1 != 0;
        self.so1_output_volume = value & 0b111;
    }

    pub fn read_sound_on_reg(&self) -> u8 {
        let bit7 = if self.enable_sound_controller { 1 << 7 } else { 0 };
        let bit3 = if self.sound_4_on { 1 << 3 } else { 0 };
        let bit2 = if self.sound_3_on { 1 << 2 } else { 0 };
        let bit1 = if self.sound_2_on { 1 << 1 } else { 0 };
        let bit0 = if self.sound_1_on { 1 << 0 } else { 0 };
        bit7 | bit3 | bit2 | bit1 | bit0
    }

    pub fn write_sound_on_reg(&mut self, value: u8) {
        self.enable_sound_controller = value & (1 << 7) != 0;
    }
}

#[derive(Debug)]
enum WaveDuty {
    Eighth,       // 12.5%
    Quarter,      // 25%
    Half,         // 50%
    ThreeQuarters // 75%
}

#[derive(Debug)]
enum EnvDir {
    Down,
    Up
}
