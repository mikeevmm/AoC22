mod emulator;
mod parser;

#[test]
fn noparsing() {
    // [ noop ] [ addx ] [... ] [ addx ] [ ... ]
    // 0        1        2      3        4
    let program = [
        emulator::Command::NoOp,
        emulator::Command::AddX(2),
        emulator::Command::AddX(-1),
    ]
    .into_iter();
    let mut cpu = emulator::Cpu::load(program);
    while !cpu.done() {
        println!("{}:{}", cpu.clock().count_finished(), cpu.x_register().value());
        cpu.finish_frame();
    }
    println!("{}:{}", cpu.clock().count_finished(), cpu.x_register().value());
    assert_eq!(cpu.x_register().value(), 1);
}

#[test]
fn onlyparsing() {
    for command in parser::parse_program("input") {
        println!("{:?}", command);
    }
}

#[test]
fn exampleinput() {
    let program = parser::parse_program("exampleinput");
    let answer = get_interesting_signal(program);
    assert_eq!(answer, 13140)
}

fn get_interesting_signal(program: impl emulator::Program) -> isize {
    let mut cpu = emulator::Cpu::load(program);
    
    let mut answer_acc = 0;
    
    // "Warm up" the first 19 cycles
    for _ in 0..19 {
        cpu.finish_frame();
    }
    
    while !cpu.done() {
        let current_tick = cpu.clock().current_frame();
        if current_tick > 220 {
            break;
        }
        if (current_tick - 20) % 40 == 0 {
            answer_acc += cpu.x_register().value() * current_tick as isize;
        }
        cpu.finish_frame();
    }
    
    answer_acc
}

#[test]
fn examplecrt() {
    let program = parser::parse_program("exampleinput");
    let mut cpu = emulator::Cpu::load(program);
    let mut crt = emulator::crt::Crt::new();
    
    while !cpu.done() {
        crt.read(&cpu);
        cpu.finish_frame();
    }
    
    let buffer = crt.buffer();
    println!("{}", buffer);
}

fn main() {
    let program = parser::parse_program("input");
    let answer = get_interesting_signal(program);
    println!("Sum of signals is {}", answer);
    
    let program = parser::parse_program("input");
    let mut cpu = emulator::Cpu::load(program);
    let mut crt = emulator::crt::Crt::new();
    
    while !cpu.done() {
        crt.read(&cpu);
        cpu.finish_frame();
    }
    
    let buffer = crt.buffer();
    println!("{}", buffer);
}
