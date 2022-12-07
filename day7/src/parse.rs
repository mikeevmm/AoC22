use crate::{Entry, Command};
use nom::bytes::complete::take_while;
use nom::character::complete::{char, digit1};
use nom::combinator::peek;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::{
    bytes::streaming::tag,
    character::complete::anychar,
    combinator::{eof, opt},
    sequence::{preceded, terminated},
};

fn next(i: &str) -> nom::IResult<&str, &str> {
    if opt(eof)(i)?.1.is_some() {
        return Ok((i, &""));
    }
    let (rest, cr) = match opt(char('\r'))(i) {
        Ok((rest, cr)) => Ok((rest, cr.is_some())),
        Err(e) => Err(e),
    }?;
    match char('\n')(rest) {
        Ok((rest, _)) => Ok((rest, if cr { &"\r\n" } else { &"\n" })),
        Err(e) => Err(e),
    }
}

fn word(i: &str) -> nom::IResult<&str, String> {
    let mut word = String::new();
    let mut rest = i;
    let mut escape = false;
    loop {
        if rest == "" {
            return Ok(("", word));
        }
        if escape {
            let (new_rest, matched) = anychar(rest)?;
            word.push(matched);
            rest = new_rest;
            escape = false;
        } else {
            let (_, peeked) = peek(anychar)(rest)?;
            if peeked.is_whitespace() {
                return Ok((rest, word));
            }
            let (new_rest, matched) = anychar(rest)?;
            if matched == '\\' {
                escape = true;
            } else {
                word.push(matched)
            }
            rest = new_rest;
        }
    }
}

fn command(i: &str) -> nom::IResult<&str, Vec<String>> {
    preceded(
        tag("$ "),
        terminated(separated_list1(char(' '), word), next),
    )(i)
}

fn directory_listing(i: &str) -> nom::IResult<&str, String> {
    preceded(tag("dir "), word)(i)
}

fn file_listing(i: &str) -> nom::IResult<&str, (u64, String)> {
    let (rest, (size, name)) = separated_pair(
        digit1,
        char(' '),
        take_while(|c: char| c.is_alphanumeric() || c == '.' || c == '_'),
    )(i)?;
    let size = str::parse::<u64>(size).expect("the number to fit in a u64");
    Ok((rest, (size, name.to_string())))
}

fn parse_command(raw: Vec<String>) -> Entry {
    let (cmd, args) = raw.split_at(1);
    match cmd[0].as_str() {
        "cd" => {
            if args.len() > 1 {
                println!("WARNING: Ignoring arguments to ls: {:?}", &args[1..]);
            }
            Entry::User(Command::Cd(args[0].to_owned()))
        },
        "ls" => {
            if args.len() > 0 {
                println!("WARNING: Ignoring arguments to ls: {:?}", args);
            }
            Entry::User(Command::Ls)
        },
        cmd => panic!("Unknown command {}", cmd),
    }
}

pub fn entry(i: &str) -> Entry {
    let (rest, entry) = command(i)
        .map(|(rest, command)| (rest, parse_command(command)))
        .or(directory_listing(i).map(|(rest, name)| (rest, Entry::Directory(name))))
        .or(file_listing(i).map(|(rest, (size, name))| (rest, Entry::File(size, name))))
        .expect("Don't know how to read a line!");
    if rest.trim().len() != 0 {
        panic!("Trailing contents: {}", rest);
    }
    entry
}
