use dmg::cpu::Cpu;

#[derive(Debug)]
pub struct Dmg {
    cpu: Cpu,
}

impl Dmg {
    pub fn new(boot: Vec<u8>) -> Dmg {
        Dmg {
            cpu: Cpu::new(boot)
        }
    }

    pub fn run(&mut self) {
        self.cpu.run();
    }

    pub fn step(&mut self) {
        self.cpu.step();
    }
}
