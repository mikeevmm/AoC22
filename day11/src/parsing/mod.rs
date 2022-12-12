/// Here be dragons. I didn't feel like the parsing was the main part of the problem, so I wasn't
/// too invested in writing really good/flexible code for this; but, obviously, the problem is very
/// parsing-heavy.

use self::chain::ParserChain;
use crate::{Monkey, Operation, Operator, RValue, Test};
use nom::{
    branch::alt,
    bytes::streaming::tag,
    character::complete::{digit1, space1},
    character::{complete::char, complete::space0},
    combinator::eof,
    error::ErrorKind,
    multi::{many1, separated_list1},
    sequence::{delimited, preceded, terminated, tuple},
    Finish, IResult, Parser,
};
use std::{fs::File, io::Read, path::Path, collections::VecDeque};

mod chain;

fn eol(i: &str) -> IResult<&str, &str> {
    alt((eof, tag("\r\n"), tag("\n")))(i)
}

fn prop_starting(i: &str) -> IResult<&str, VecDeque<usize>> {
    let (rest, items) = preceded(
        tag("Starting items: "),
        separated_list1(
            char(','),
            delimited(space0, digit1, space0)
                .map(|v: &str| v.parse::<usize>().expect("digits to be parsable as usize")),
        ),
    )(i)?;
    Ok((rest, items.into()))
}

fn parse_formula(i: &str) -> IResult<&str, Operation> {
    fn parse_rvalue(i: &str) -> IResult<&str, RValue> {
        alt((
            tag("old").map(|_| RValue::Old),
            digit1.map(|v: &str| {
                RValue::Literal(
                    v.parse::<usize>()
                        .expect("to be able to parse matched digits"),
                )
            }),
        ))(i.trim())
    }

    fn parse_operator(i: &str) -> IResult<&str, Operator> {
        let (rest, operator) = alt((char('+'), char('-'), char('*')))(i.trim())?;
        let operator = match operator {
            '+' => Operator::Add,
            '-' => Operator::Sub,
            '*' => Operator::Mul,
            _ => return Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Fail))),
        };
        Ok((rest, operator))
    }

    // Hey, at least they're not making me parse a general calculator.
    // From inspection, the input follows the following grammar:
    //
    //  <expression> ::= 'new = old' <operator> <rvalue>
    //  <operator>   ::= '+' | '-' | '*'
    //  <rvalue>     ::= 'old' | <number>
    //  <number>     ~= [0-9]+
    let (rest, (operator, rvalue)) =
        preceded(tag("new = old"), tuple((parse_operator, parse_rvalue)))(i)?;
    let equation = Operation { operator, rvalue };

    Ok((rest, equation))
}

fn prop_operation(i: &str) -> IResult<&str, Operation> {
    preceded(tag("Operation: "), parse_formula)(i)
}

fn prop_test(indent: &str) -> impl Fn(&str) -> IResult<&str, Test> + '_ {
    move |i: &str| {
        let (rest, divider) = preceded(
            tag("Test: divisible by "),
            digit1.map(|v: &str| v.parse::<usize>().expect("digits to be parsable as usize")),
        )(i)?;
        let (rest, if_true) = eol
            .chain(tag(indent))
            .chain(tag("  "))
            .chain(tag("If true: throw to monkey "))
            .chain(digit1)
            .map(|v: &str| v.parse::<usize>().expect("digits to be parsable as usize"))
            .parse(rest)?;
        let (rest, if_false) = eol
            .chain(tag(indent))
            .chain(tag("  "))
            .chain(tag("If false: throw to monkey "))
            .chain(digit1)
            .map(|v: &str| v.parse::<usize>().expect("digits to be parsable as usize"))
            .parse(rest)?;
        let test = Test {
            divider,
            if_true,
            if_false,
        };
        Ok((rest, test))
    }
}

fn parse_monkey(i: &str) -> IResult<&str, (usize, Monkey)> {
    let (i, monkey_i) = delimited(
        tag("Monkey "),
        digit1.map(|v: &str| v.parse::<usize>().expect("digits to be parsable as usize")),
        tag(":"),
    )(i)?;
    let (i, _char) = eol(i)?;
    let (i, indent) = space1(i)?;
    let (i, items) = terminated(prop_starting, eol).parse(i)?;
    let (i, operation) = delimited(tag(indent), prop_operation, eol).parse(i)?;
    let (i, test) = preceded(tag(indent), prop_test(indent)).parse(i)?;
    let monkey = Monkey {
        items,
        operation,
        test,
    };
    Ok((i, (monkey_i, monkey)))
}

pub fn parse<P: AsRef<Path>>(file: P) -> Vec<Monkey> {
    let mut contents = String::new();
    File::open(file)
        .expect("input file to exist and be readable")
        .read_to_string(&mut contents)
        .expect("to be able to read input file into memory");

    let (_, parse_output) =
        separated_list1(many1(space0.chain(eol)), parse_monkey)(contents.trim())
            .finish()
            .expect("to correctly parse the input file");
    let (monkey_indices, monkeys): (Vec<usize>, Vec<Monkey>) = parse_output.into_iter().unzip();

    for (i, parsed_i) in monkey_indices.into_iter().enumerate() {
        if i != parsed_i {
            panic!(
                "Correctly parsed monkeys, but the parsed indices do not match expected indices!"
            );
        }
    }

    monkeys
}
