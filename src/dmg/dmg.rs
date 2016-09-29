use dmg::cpu::Cpu;
use dmg::interconnect::Interconnect;
use Color;

pub struct Dmg {
    cpu: Cpu,
    interconnect: Interconnect,
}

impl Dmg {
    pub fn new(boot: Box<[u8]>, rom: Box<[u8]>) -> Dmg {
        Dmg {
            cpu: Cpu::new(),
            interconnect: Interconnect::new(boot, rom),
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn framebuffer(&self) -> &[Color] {
        self.interconnect.framebuffer()
    }

    pub fn interconnect(&self) -> &Interconnect {
        &self.interconnect
    }

    pub fn step(&mut self) -> usize {
        let int_cycles = self.proc_interrupts();
        let cycles = self.cpu.step(&mut self.interconnect);
        for _ in 0..(cycles + int_cycles) {
            self.interconnect.step(4);
        }
        cycles + int_cycles
    }

    fn proc_interrupts(&mut self) -> usize {
        let int_flags = self.interconnect.read_byte(0xFF0F);
        let en_flags = self.interconnect.read_byte(0xFFFF);
        for bit in 0..5 {
            let flagged = int_flags >> bit & 0b1 != 0;
            let enabled = en_flags >> bit & 0b1 != 0;
            if flagged && enabled {
                self.cpu.halted = false;
                if self.cpu.halt_no_jump {
                    self.cpu.halt_no_jump = false;
                    return 0;
                }
                if self.cpu.ime {
                    self.interconnect.write_byte(0xFF0F, int_flags & !(1 << bit));
                    return self.interrupt(bit);
                }
            }
        }
        0
    }

    fn interrupt(&mut self, flag_bit: u8) -> usize {
        let addr = match flag_bit {
            0 => 0x40,
            1 => 0x48,
            2 => 0x50,
            3 => 0x58,
            4 => 0x60,
            _ => unreachable!()
        };
        self.cpu.interrupt(addr, &mut self.interconnect)
    }
}
