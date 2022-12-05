use std::{
    fs::File,
    io::{BufRead, BufReader, Lines, Read},
    iter::Peekable,
    marker::PhantomData,
    str::Chars, collections::VecDeque,
};

/// By Neil Roberts (https://stackoverflow.com/a/67573987). (Adapted)
///
/// Lets us get characters directly.
struct CharGetter {
    // Buffer containing one line of input at a time
    input_buf: String,
    // The byte position within input_buf of the next character to
    // return.
    input_pos: usize,
}

impl CharGetter {
    fn next(&mut self) -> Option<char> {
        // Get an iterator over the string slice starting at the
        // next byte position in the string
        let mut input_pos = self.input_buf[self.input_pos..].chars();

        // Try to get a character from the temporary iterator
        match input_pos.next() {
            // If there is still a character left in the input
            // buffer then we can just return it immediately.
            Some(n) => {
                // Move the position along by the number of bytes
                // that this character occupies in UTF-8
                self.input_pos += n.len_utf8();
                Some(n)
            }
            // Otherwise there's nothing left in this line
            None => None,
        }
    }
}

/// A struct that takes a `BufReader` and implements `Iterator` over its `char`s.
struct BufReaderIter<R: Read> {
    lines: Lines<BufReader<R>>,
    lines_iter: Option<CharGetter>,
    exhausted: bool,
}

impl<R: Read> BufReaderIter<R> {
    fn new(reader: BufReader<R>) -> Self {
        BufReaderIter {
            lines: reader.lines(),
            lines_iter: None,
            exhausted: false,
        }
    }
}

impl<R: Read> Iterator for BufReaderIter<R> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        if self.lines_iter.is_none() {
            let next_line = self
                .lines
                .next()
                .map(|l| l.expect("every line to be readable"));
            if next_line.is_none() {
                self.exhausted = true;
                return None;
            }
            let next_line = next_line.unwrap();
            self.lines_iter = Some(CharGetter {
                input_buf: next_line,
                input_pos: 0,
            });
        }

        let next_char = self.lines_iter.as_mut().unwrap().next();
        if next_char.is_none() {
            self.lines_iter = None;
            return Some('\n');
        }
        return Some(next_char.unwrap());
    }
}

/// A simple parser over anything that reads `char`s.
struct Parser<'r, R>
where
    R: Iterator<Item = char> + 'r,
{
    inner: Peekable<R>,
    lifetime: PhantomData<&'r R>,
}

impl<'over, R: Read + 'over> Parser<'over, BufReaderIter<R>> {
    fn from_reader(reader: BufReader<R>) -> Self {
        Parser {
            inner: BufReaderIter::new(reader).peekable(),
            lifetime: PhantomData,
        }
    }
}

impl<'over> Parser<'over, Chars<'over>> {
    fn from_chars(chars: Chars<'over>) -> Self {
        Parser {
            inner: chars.peekable(),
            lifetime: PhantomData,
        }
    }
}

impl<'r, R> Parser<'r, R>
where
    R: Iterator<Item = char> + 'r,
{
    fn eof(&mut self) -> bool {
        self.inner.peek().is_none()
    }

    fn match_(&mut self, to_match: char) {
        match self.inner.next() {
            Some(c) => {
                if c != to_match {
                    panic!("unexpected char {}", c);
                }
            }
            None => panic!("tried to match {} at end of file", to_match),
        }
    }

    fn try_match(&mut self, to_match: char) -> bool {
        if let Some(&c) = self.inner.peek() {
            if c == to_match {
                self.inner.next().expect("to match the peeked character");
                return true;
            }
        }
        false
    }

    fn try_match_predicate<F: Fn(&char) -> bool>(&mut self, predicate: F) -> bool {
        if let Some(c) = self.inner.peek() {
            if predicate(c) {
                self.inner.next().expect("to match the peeked character");
                return true;
            }
        }
        false
    }

    fn match_str(&mut self, to_match: &str) {
        for c in to_match.chars() {
            self.match_(c);
        }
    }

    fn eat_predicate<F: Fn(&char) -> bool>(&mut self, predicate: F) {
        while let Some(c) = self.inner.peek() {
            if predicate(c) {
                self.inner.next().expect("to match the peeked character");
            } else {
                break;
            }
        }
    }
    
    fn peek_predicate<F: Fn(&char) -> bool>(&mut self, predicate: F) -> bool {
        if let Some(c) = self.inner.peek() {
            predicate(c)
        } else {
            false
        }
    }

    fn number(&mut self) -> u32 {
        let mut collected = String::new();
        while let Some(&next) = self.inner.peek() {
            if next.is_numeric() {
                self.inner.next().expect("to consume the peeked character");
                collected.push(next);
            } else {
                break;
            }
        }
        collected.parse::<u32>().expect("to match at least a digit")
    }
    
    fn take(&mut self) -> char {
        self.inner.next().expect("a character to be left")
    }
}

#[derive(Clone)]
struct Piles {
    piles: Vec<VecDeque<char>>,
}

impl Piles {
    fn new(count: usize) -> Self {
        Piles {
            piles: vec![VecDeque::new(); count],
        }
    }

    fn put(&mut self, on: usize, value: char) {
        self.piles[on].push_front(value);
    }
    
    fn shuffle(&mut self, count: u32, from: usize, to: usize) {
        for _ in 0..count {
            let to_move = self.piles[from].pop_back().expect("enough items to exist to be moved");
            self.piles[to].push_back(to_move);
        }
    }
    
    fn move_(&mut self, count: u32, from: usize, to: usize) {
        let size = self.piles[from].len();
        let moved: Vec<char> = self.piles[from].drain((size - count as usize)..).collect();
        for element in moved {
            self.piles[to].push_back(element);
        }
    }
    
    fn output(self) {
        for mut pile in self.piles {
            print!("{}", pile.pop_back().unwrap_or(' '));
        }
        print!("\n");
    }
}

fn main() {
    // Read the number of piles.
    // This is the easiest way I could think of doing it.
    let number_of_piles: usize = {
        let input = BufReader::new(File::open("input").expect("input to exist and be readable"));
        let mut number = 0;
        for line in input.lines().map(|l| l.expect("line to be readable")) {
            let mut parser = Parser::from_chars(line.chars());
            parser.eat_predicate(|c| c.is_whitespace());
            if parser.try_match_predicate(|c| c.is_numeric()) {
                number = 1;
                while {
                    parser.eat_predicate(|c| c.is_whitespace());
                    parser.try_match_predicate(|c| c.is_numeric())
                } {
                    number += 1;
                }
                break;
            }
        }
        number
    };

    // Knowing the number of piles, parse the elements of the piles
    let mut piles = Piles::new(number_of_piles);
    let input = BufReader::new(File::open("input").expect("input to exist and be readable"));
    let mut parser = Parser::from_reader(input);

    while {
        parser.eat_predicate(|c| c.is_whitespace());
        !parser.peek_predicate(|c| c.is_numeric())
    } {
        for pile in 0..number_of_piles {
            if parser.try_match('[') {
                piles.put(pile, parser.take());
                parser.match_(']');
            } else {
                parser.match_str("   ");
            }

            if !parser.try_match(' ') {
                parser.try_match('\r');
                parser.match_('\n');
            }
        }
    }
    
    // Eat until the first line of "move"
    parser.eat_predicate(|&c| c != 'm');
    
    // Perform the moves.
    let mut crate_mover_9000 = piles.clone();
    let mut crate_mover_9001 = piles;

    while !parser.eof() {
        parser.match_str("move ");
        let count = parser.number();
        parser.match_str(" from ");
        let from = parser.number() - 1;
        parser.match_str(" to ");
        let to = parser.number() - 1;
        parser.eat_predicate(|c| c.is_whitespace());
        crate_mover_9000.shuffle(count, from as usize, to as usize);
        crate_mover_9001.move_(count, from as usize, to as usize);
    }
    
    println!("Part 1 answer: ");
    crate_mover_9000.output();
    println!("Part 2 answer: ");
    crate_mover_9001.output();
}
