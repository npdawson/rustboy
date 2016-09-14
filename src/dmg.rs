use cpu::Cpu;

#[derive(Debug)]
pub struct Dmg {
    cpu: Cpu,
}

impl Dmg {
    pub fn new(rom: Vec<u8>) -> Dmg {
        Dmg {
            cpu: Cpu::new(rom)
        }
    }

    pub fn run(&mut self) {
        self.cpu.run();
    }

    pub fn step(&mut self) {
        self.cpu.step();
    }
}
