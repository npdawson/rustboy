use super::Opcode;

pub struct Instruction {
    opcode: Opcode
}

impl Instruction {
    pub fn new(op: Opcode) -> Instruction {
        Instruction {
            opcode: op,
        }
    }

    pub fn opcode(&self) -> Opcode {
        self.opcode
    }
}
