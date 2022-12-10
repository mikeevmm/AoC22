use crate::emulator::{Command, Program};
use std::{
    fs::File,
    io::{BufRead, BufReader, Lines, Read},
    path::Path,
};

pub struct ParsedProgram<R: Read> {
    inner: Lines<BufReader<R>>,
}

impl<R: Read> ParsedProgram<R> {
    fn new(inner: BufReader<R>) -> Self {
        ParsedProgram {
            inner: inner.lines(),
        }
    }
}

impl<R: Read> Iterator for ParsedProgram<R> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.inner.next()?.expect("to be able to read line");
        let mut words = line.split_whitespace();
        match words.next()? {
            "addx" => {
                let amount = words.next().expect("addx to be followed by a number");
                let amount = amount
                    .parse::<isize>()
                    .expect("word after addx to be a number");
                if let Some(rest) = words.next() {
                    panic!("Trailing word after addx command: {}", rest);
                }
                Some(Command::AddX(amount))
            }
            "noop" => Some(Command::NoOp),
            other => {
                panic!("Cannot parse {} command", other)
            }
        }
    }
}

pub fn parse_program<P: AsRef<Path>>(from: P) -> impl Program {
    let input = BufReader::new(File::open(from).expect("path to exist and be readable"));
    ParsedProgram::new(input)
}
