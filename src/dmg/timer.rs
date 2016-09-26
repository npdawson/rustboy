#[derive(Debug)]
pub struct Timer {
    divider_reg: u16, // FF04
    counter: u16,    // FF05
    pub modulo: u8,  // FF06
    // FF07 Timer Control
    enabled: bool,
    input_clock: Clock,
    tim_cycle_count: usize
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider_reg: 0,
            counter: 0,
            modulo: 0,
            enabled: false,
            input_clock: Clock::C4KHz,
            tim_cycle_count: 0,
        }
    }

    pub fn step(&mut self, cycles: usize) -> bool {
        // divider always runs
        self.divider_reg = self.divider_reg.wrapping_add(cycles as u16);

        // timer runs when enabled
        if self.enabled {
            // TODO connect timer to div bits
            // TODO Falling edge detector
            self.tim_cycle_count += cycles;
            let max = match self.input_clock {
                Clock::C4KHz => 1024,
                Clock::C256KHz => 16,
                Clock::C64KHz => 64,
                Clock::C16KHz => 256,
            };
            if self.tim_cycle_count >= max {
                self.tim_cycle_count -= max;
                self.counter.wrapping_add(1);
                if self.counter == 0x101 {
                    self.counter = self.modulo as u16;
                    return true;
                }
            }
        }
        false
    }

    pub fn read_div_reg(&self) -> u8 {
        (self.divider_reg >> 8) as u8
    }

    pub fn write_div_reg(&mut self) {
        self.divider_reg = 0;
    }

    pub fn read_counter(&self) -> u8 {
        self.counter as u8
    }

    pub fn write_counter(&mut self, value: u8) {
        self.counter = value as u16;
    }

    pub fn read_timer_control(&self) -> u8 {
        let bit2 = if self.enabled { 1 << 2 } else { 0 };
        let bit10 = match self.input_clock {
            Clock::C4KHz => 0b00,
            Clock::C256KHz => 0b01,
            Clock::C64KHz => 0b10,
            Clock::C16KHz => 0b11
        };
        bit2 | bit10
    }

    pub fn write_timer_control(&mut self, value: u8) {
        self.enabled = value >> 2 & 1 != 0;
        self.input_clock = match value & 0b11 {
            00 => Clock::C4KHz,
            01 => Clock::C256KHz,
            10 => Clock::C64KHz,
            11 => Clock::C16KHz,
            _ => unreachable!()
        };
    }
}

#[derive(Debug)]
enum Clock {
    C4KHz,
    C256KHz,
    C64KHz,
    C16KHz
}
