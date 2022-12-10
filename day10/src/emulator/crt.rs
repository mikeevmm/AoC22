use super::{Cpu, Program};
use itertools::Itertools;

pub struct Crt {
    buffer: String,
}

impl Crt {
    pub fn new() -> Self {
        Crt {
            buffer: String::new(),
        }
    }

    pub fn read<P: Program>(&mut self, cpu: &Cpu<P>) {
        let x_position = cpu.x_register().value();
        let crt_position = (self.buffer.len() % 40) as isize;
        if crt_position.abs_diff(x_position) <= 1 {
            self.buffer.push('#');
        } else {
            self.buffer.push('.');
        }
    }

    pub fn buffer(self) -> String {
        let lines = (self.buffer.len() - 1) / 40 + 1;
        (1..=lines).map(|i| &self.buffer[(i - 1) * 40..i * 40]).intersperse("\n").collect()
    }

    pub fn print_line(&self) {
        println!("{}", &self.buffer[((self.buffer.len() - 1) / 40) * 40..])
    }
}
