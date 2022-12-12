use std::collections::VecDeque;

mod parsing;
mod partialsort;

#[derive(Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
}

#[derive(Debug)]
pub enum RValue {
    Old,
    Literal(usize),
}

#[derive(Debug)]
pub struct Operation {
    operator: Operator,
    rvalue: RValue,
}

#[derive(Debug)]
pub struct Test {
    divider: usize,
    if_true: usize,
    if_false: usize,
}

#[derive(Debug)]
pub struct Monkey {
    items: VecDeque<usize>,
    operation: Operation,
    test: Test,
}

impl Operation {
    fn operate(&self, in_value: usize) -> usize {
        let lvalue = in_value;
        let rvalue = match self.rvalue {
            RValue::Old => in_value,
            RValue::Literal(literal) => literal,
        };
        match self.operator {
            Operator::Add => lvalue + rvalue,
            Operator::Sub => lvalue - rvalue,
            Operator::Mul => lvalue * rvalue,
        }
    }
    
    fn operate_under_ring(&self, in_value: usize, ring: usize) -> usize {
        let lvalue = in_value;
        let rvalue = match self.rvalue {
            RValue::Old => in_value,
            RValue::Literal(literal) => literal,
        };
        match self.operator {
            Operator::Add => ((lvalue % ring) + (rvalue % ring)) % ring,
            Operator::Sub => ((lvalue % ring) - (rvalue % ring)) % ring,
            Operator::Mul => ((lvalue % ring) * (rvalue % ring)) % ring,
        }
        
    }
}

impl Test {
    fn test(&self, in_value: usize) -> usize {
        if in_value % self.divider == 0 {
            self.if_true
        } else {
            self.if_false
        }
    }
}

impl Monkey {
    fn throw_to(&self, in_value: usize) -> usize {
        self.test.test(in_value)
    }
}

struct MonkeyBusiness {
    monkeys: Vec<Monkey>,
    monkey_number: usize,
    ring: usize,
    inspection: Vec<usize>,
}

impl MonkeyBusiness {
    fn new(monkeys: Vec<Monkey>) -> Self {
        let size = monkeys.len();
        let ring = monkeys.iter().fold(1, |acc, m| acc * m.test.divider);
        MonkeyBusiness {
            monkeys,
            monkey_number: size,
            ring,
            inspection: vec![0; size],
        }
    }

    fn round(&mut self) {
        for monkey_index in 0..self.monkey_number {
            loop {
                let monkey = &mut self.monkeys[monkey_index];
                let worry_level = monkey.items.pop_front();
                if worry_level.is_none() {
                    break;
                }
                let worry_level = monkey.operation.operate(worry_level.unwrap());
                let worry_level = worry_level / 3;
                let throw_to = monkey.throw_to(worry_level);
                drop(monkey);
                self.monkeys[throw_to].items.push_back(worry_level);
                self.inspection[monkey_index] += 1;
            }
        }
    }
    
    fn unbounded_round(&mut self) {
        for monkey_index in 0..self.monkey_number {
            loop {
                let monkey = &mut self.monkeys[monkey_index];
                let worry_level = monkey.items.pop_front();
                if worry_level.is_none() {
                    break;
                }
                let worry_level = monkey.operation.operate_under_ring(worry_level.unwrap(), self.ring);
                let throw_to = monkey.throw_to(worry_level);
                drop(monkey);
                self.monkeys[throw_to].items.push_back(worry_level);
                self.inspection[monkey_index] += 1;
            }
        }
    }
    
    fn monkey_business(&self) -> usize {
        let mut inspections = self.inspection.clone();
        partialsort::partial_sort(&mut inspections, 2);
        inspections[0] * inspections[1]
    }
    
    fn monkeys(self) -> Vec<Monkey> {
        self.monkeys
    }

    fn debug_inspections(&self) {
        println!("{:?}", self.inspection)
    }
}

#[test]
fn parsing() {
    println!("{:?}", parsing::parse("exampleinput"));
}

#[test]
fn exampleinput1() {
    let monkeys = parsing::parse("exampleinput");
    let mut monkey_business = MonkeyBusiness::new(monkeys);

    for _round in 0..20 {
        monkey_business.round();
    }
    
    assert_eq!(monkey_business.monkey_business(), 10605);
    
    let monkeys = monkey_business.monkeys();
    assert_eq!(&monkeys[0].items, &[10, 12, 14, 26, 34]);
    assert_eq!(&monkeys[1].items, &[245, 93, 53, 199, 115]);
    assert_eq!(&monkeys[2].items, &[]);
    assert_eq!(&monkeys[3].items, &[]);
}

#[test]
fn exampleinput2() {
    let monkeys = parsing::parse("exampleinput");
    let mut monkey_business = MonkeyBusiness::new(monkeys);

    for _round in 0..10_000 {
        monkey_business.unbounded_round();
    }
    
    assert_eq!(monkey_business.monkey_business(), 2713310158);
}

fn part1() {
    let monkeys = parsing::parse("input");
    let mut monkey_business = MonkeyBusiness::new(monkeys);

    for _round in 0..20 {
        monkey_business.round();
    }
    
    println!("Ooh ooh ahh: {}", monkey_business.monkey_business());
}

fn part2() {
    let monkeys = parsing::parse("input");
    let mut monkey_business = MonkeyBusiness::new(monkeys);

    for _round in 0..10_000 {
        monkey_business.unbounded_round();
    }
    
    println!("Ooh ooh ahh: {}", monkey_business.monkey_business());
}

fn main() {
    part1();
    part2();
}
