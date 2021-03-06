#[derive(Debug)]
pub struct Apu {
    channel1_sweep_time: u8,
    channel1_sweep_direction: Sweep,
    channel1_sweep_shift: u8,

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

    channel2_wave: WaveDuty,
    channel2_length: u8,
    channel2_envelope_volume: u8,
    channel2_envelope_direction: EnvDir,
    channel2_envelope_sweeps: u8,

    channel2_frequency: u16,
    channel2_counter_consecutive: bool,

    channel3_enable: bool,
    channel3_length: u8,
    channel3_volume: u8,
    channel3_frequency: u16,
    channel3_counter_consecutive: bool,
    wave_pattern_ram: Box<[u8]>,

    channel4_length: u8,
    channel4_envelope_volume: u8,
    channel4_envelope_direction: EnvDir,
    channel4_envelope_sweeps: u8,
    channel4_shift_freq: u8,
    channel4_counter_step: bool,
    channel4_div_ratio: u8,
    channel4_counter_consecutive: bool,

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
            channel1_sweep_time: 0,
            channel1_sweep_direction: Sweep::Inc,
            channel1_sweep_shift: 0,

            channel1_wave: WaveDuty::Half,
            channel1_length: 0x3F,

            channel1_envelope_volume: 0xF,
            channel1_envelope_direction: EnvDir::Down,
            channel1_envelope_sweeps: 3,

            channel1_frequency: 0x70,
            channel1_counter_consecutive: false,

            channel2_wave: WaveDuty::Half,
            channel2_length: 0x3F,

            channel2_envelope_volume: 0x0,
            channel2_envelope_direction: EnvDir::Down,
            channel2_envelope_sweeps: 0,

            channel2_frequency: 0x70,
            channel2_counter_consecutive: false,

            channel3_enable: false,
            channel3_length: 0,
            channel3_volume: 0,
            channel3_frequency: 0,
            channel3_counter_consecutive: false,
            wave_pattern_ram: vec![0; 0x10].into_boxed_slice(),

            channel4_length: 0,
            channel4_envelope_volume: 0,
            channel4_envelope_direction: EnvDir::Down,
            channel4_envelope_sweeps: 0,
            channel4_shift_freq: 0,
            channel4_counter_step: false,
            channel4_div_ratio: 0,
            channel4_counter_consecutive: false,

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

    pub fn read_chan1_sweep(&self) -> u8 {
        let bits6to4 = self.channel1_sweep_time << 4;
        let bit3 = match self.channel1_sweep_direction {
            Sweep::Inc => 0,
            Sweep::Dec => 1
        };
        bits6to4 | bit3 | (self.channel1_sweep_shift & 0b111)

    }

    pub fn write_chan1_sweep(&mut self, value: u8) {
        self.channel1_sweep_time = value >> 4;
        self.channel1_sweep_direction = match value >> 3 & 1 {
            0 => Sweep::Inc,
            _ => Sweep::Dec
        };
        self.channel1_sweep_shift = value & 0b111;
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
        let hi = self.channel1_frequency & 0x700;
        self.channel1_frequency = hi | value as u16;
    }

    pub fn read_chan1_freq_hi(&self) -> u8 {
        if self.channel1_counter_consecutive { 1 << 6 } else { 0 }
    }

    pub fn write_chan1_freq_hi(&mut self, value: u8) {
        if value >> 7 != 0 {
            self.sound_1_on = true;
        }
        self.channel1_counter_consecutive = value >> 6 & 1 != 0;
        let freq_hi = ((value as u16) & 0b111) << 8;
        self.channel1_frequency = self.channel1_frequency & 0xFF | freq_hi;
    }

    pub fn read_chan2_wavelength(&self) -> u8 {
        let bit76 = match self.channel2_wave {
            WaveDuty::Eighth  => 0b00 << 6,
            WaveDuty::Quarter => 0b01 << 6,
            WaveDuty::Half    => 0b10 << 6,
            WaveDuty::ThreeQuarters => 0x11 << 6,
        };
        bit76 | self.channel2_length
    }

    pub fn write_chan2_wavelength(&mut self, value: u8) {
        self.channel2_wave = match value >> 6 {
            0b00 => WaveDuty::Eighth,
            0b01 => WaveDuty::Quarter,
            0b10 => WaveDuty::Half,
            0b11 => WaveDuty::ThreeQuarters,
            _ => unreachable!()
        };
        self.channel2_length = value & 0x3F;
    }


    pub fn read_chan2_envelope(&self) -> u8 {
        let direction = match self.channel2_envelope_direction {
            EnvDir::Down => 0,
            EnvDir::Up   => 1
        };
        self.channel2_envelope_volume << 4
            | direction
            | self.channel2_envelope_sweeps
    }

    pub fn write_chan2_envelope(&mut self, value: u8) {
        self.channel2_envelope_volume = value >> 4;
        self.channel2_envelope_direction = match value >> 3 & 1 {
            0 => EnvDir::Down,
            _ => EnvDir::Up
        };
        self.channel2_envelope_sweeps = value & 0b111;
    }

    pub fn write_chan2_freq_lo(&mut self, value: u8) {
        let hi = self.channel2_frequency & 0x700;
        self.channel2_frequency = hi | value as u16;
    }

    pub fn read_chan2_freq_hi(&self) -> u8 {
        if self.channel2_counter_consecutive { 1 << 6 } else { 0 }
    }

    pub fn write_chan2_freq_hi(&mut self, value: u8) {
        if value >> 7 != 0 {
            self.sound_2_on = true;
        }
        self.channel2_counter_consecutive = value >> 6 & 1 != 0;
        let freq_hi = ((value as u16) & 0b111) << 8;
        self.channel2_frequency = self.channel2_frequency & 0xFF | freq_hi;
    }

    pub fn read_chan3_enable(&self) -> u8 {
        if self.channel3_enable {
            1 << 7
        } else {
            0
        }
    }

    pub fn write_chan3_enable(&mut self, value: u8) {
        self.channel3_enable = value >> 7 != 0;
    }

    pub fn read_chan3_length(&self) -> u8 {
        self.channel3_length
    }

    pub fn write_chan3_length(&mut self, value: u8) {
        self.channel3_length = value;
    }

    pub fn read_chan3_volume(&self) -> u8 {
        self.channel3_volume & 0b01100000
    }

    pub fn write_chan3_volume(&mut self, value: u8) {
        self.channel3_volume = value & 0b01100000;
    }

    pub fn write_chan3_freq_lo(&mut self, value: u8) {
        let hi = self.channel3_frequency & 0x700;
        self.channel3_frequency = hi | value as u16;
    }

    pub fn read_chan3_freq_hi(&self) -> u8 {
        if self.channel3_counter_consecutive {1 << 6} else {0}
    }

    pub fn write_chan3_freq_hi(&mut self, value: u8) {
        if value >> 7 != 0 {
            self.sound_3_on = true;
        }
        self.channel3_counter_consecutive = value >> 6 & 1 != 0;
        let freq_hi = ((value as u16) & 0b111) << 8;
        let freq_lo = self.channel3_frequency & 0xFF;
        self.channel2_frequency = freq_hi | freq_lo;
    }

    pub fn read_wave_pattern_ram(&self, offset: usize) -> u8 {
        self.wave_pattern_ram[offset]
    }

    pub fn write_wave_pattern_ram(&mut self, offset: usize, value: u8) {
        self.wave_pattern_ram[offset] = value;
    }

    pub fn read_chan4_length(&self) -> u8 {
        self.channel4_length & 0x3F
    }

    pub fn write_chan4_length(&mut self, value: u8) {
        self.channel4_length = value & 0x3F;
    }

    pub fn read_chan4_envelope(&self) -> u8 {
        let direction = match self.channel4_envelope_direction {
            EnvDir::Down => 0,
            EnvDir::Up   => 1
        };
        self.channel4_envelope_volume << 4
            | direction
            | self.channel4_envelope_sweeps
    }

    pub fn write_chan4_envelope(&mut self, value: u8) {
        self.channel4_envelope_volume = value >> 4;
        self.channel4_envelope_direction = match value >> 3 & 1 {
            0 => EnvDir::Down,
            _ => EnvDir::Up
        };
        self.channel4_envelope_sweeps = value & 0b111;
    }

    pub fn read_chan4_polycounter(&self) -> u8 {
        let bits7to4 = self.channel4_shift_freq << 4;
        let bit3 = if self.channel4_counter_step {1 << 3} else {0};
        let bits2to0 = self.channel4_div_ratio;
        bits7to4 | bit3 | bits2to0
    }

    pub fn write_chan4_polycounter(&mut self, value: u8) {
        self.channel4_shift_freq = value >> 4;
        self.channel4_counter_step = value >> 3 & 1 != 0;
        self.channel4_div_ratio = value & 0b111;
    }

    pub fn read_chan4_counter_consec(&self) -> u8 {
        if self.channel4_counter_consecutive {
            1 << 6
        } else {
            0
        }
    }

    pub fn write_chan4_counter_consec(&mut self, value: u8) {
        if value >> 7 != 0 {
            self.sound_4_on = true;
        }
        self.channel4_counter_consecutive = value >> 6 & 1 != 0;
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

#[derive(Debug)]
enum Sweep {
    Inc,
    Dec
}
