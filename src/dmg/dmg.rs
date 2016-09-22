use dmg::cpu::Cpu;
use dmg::mmu::Mmu;

#[derive(Debug)]
pub struct Dmg {
    cpu: Cpu,
    mmu: Mmu,
}

impl Dmg {
    pub fn new(boot: Vec<u8>) -> Dmg {
        Dmg {
            cpu: Cpu::new(),
            mmu: Mmu::new(boot)
        }
    }

    pub fn step(&mut self) {
        let cycles = self.cpu.step(&mut self.mmu);
        let int_cycles = self.proc_interrupts();
        self.mmu.step_gpu(cycles + int_cycles);
    }

    fn proc_interrupts(&mut self) -> usize {
        if self.cpu.ime {
            let int_flags = self.mmu.read_byte(0xFF0F);
            let en_flags = self.mmu.read_byte(0xFFFF);
            for bit in 0..5 {
                let flagged = int_flags >> bit & 0b1 != 0;
                let enabled = en_flags >> bit & 0b1 != 0;
                if flagged && enabled {
                    self.cpu.halted = false;
                    // unflag interrupt
                    self.mmu.write_byte(0xFF0F, int_flags & !(1 << bit));
                    // interrupt CPU
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
        self.cpu.interrupt(addr, &mut self.mmu)
    }
}
