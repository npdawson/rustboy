mod command;

use std::io::{stdin, stdout};
use std::io::prelude::*;
use std::borrow::Cow;

use dmg::Dmg;
use dmg::cpu::Instruction;
use dmg::cpu::Opcode::*;
use dmg::mem_map;
use dmg::mem_map::Addr::*;
use self::command::Command;

pub struct Debugger {
    dmg: Dmg,

    last_command: Option<Command>,
}

impl Debugger {
    pub fn new(dmg: Dmg) -> Debugger {
        Debugger {
            dmg: dmg,

            last_command: None,
        }
    }

    pub fn run(&mut self) {
        loop {
            print!("rustboy> ");
            stdout().flush().unwrap();

            let command = match (read_stdin().parse(), self.last_command) {
                (Ok(Command::Repeat), Some(c)) => Ok(c),
                (Ok(Command::Repeat), None) => Err("No last command".into()),
                (Ok(c), _) => Ok(c),
                (Err(e), _) => Err(e),
            };

            match command {
                Ok(Command::Step(count)) => self.step(count),
                Ok(Command::Exit) => break,
                Ok(Command::Repeat) => unreachable!(),
                Err(ref e) => println!("{}", e),
            }

            self.last_command = command.ok();
        }
    }

    pub fn step(&mut self, count: usize) {
        for _ in 0..count {
            let current_pc = self.dmg.cpu().current_pc();
            let addr = mem_map::map_addr(current_pc);
            let instr = Instruction::new(match addr {
                Rom(offset) => self.dmg.interconnect().read_byte(offset as u16),
                _ => panic!("Debugger can't inspect address: {:?}", addr),
            });

            // println!("{:018x}: {}", current_pc, instr);

            self.dmg.step();
        }
    }
}

fn read_stdin() -> String {
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().into()
}
