use std::{
    fs::File,
    io::{BufRead, BufReader},
    iter::Peekable,
    str::Chars,
};

struct TinyParser<'over> {
    inner: Peekable<Chars<'over>>,
}

impl<'over> TinyParser<'over> {
    fn new(content: &'over str) -> Self {
        TinyParser {
            inner: content.chars().peekable(),
        }
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
}

type SizeOrderedRange<'r> = (&'r Range, &'r Range);

struct Range {
    start: u32,
    end: u32,
}

impl Range {
    fn new(start: u32, end: u32) -> Self {
        debug_assert!(end >= start);
        Range { start, end }
    }

    fn parser(parser: &mut TinyParser) -> Self {
        let start = parser.number();
        parser.match_('-');
        let end = parser.number();
        Self::new(start, end)
    }

    fn intersects(&self, other: &Self) -> bool {
        if self.start > other.end {
            return false;
        }
        if self.end < other.start {
            return false;
        }
        true
    }

    fn contains(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn size(&self) -> u32 {
        self.end - self.start
    }

    /// Returns the references in ascending order of size.
    fn size_pair<'a>(&'a self, other: &'a Self) -> SizeOrderedRange<'a> {
        if self.size() < other.size() {
            (self, other)
        } else {
            (other, self)
        }
    }
}

trait ForSizeOrderedRange {
    fn containing(&self) -> bool;
}

impl ForSizeOrderedRange for SizeOrderedRange<'_> {
    fn containing(&self) -> bool {
        self.1.contains(self.0)
    }
}

#[test]
fn range_intersections() {
    assert_eq!(Range::new(6, 8).intersects(&Range::new(5, 9)), true);
    assert_eq!(Range::new(6, 8).intersects(&Range::new(5, 6)), true);
    assert_eq!(Range::new(6, 8).intersects(&Range::new(5, 7)), true);
    assert_eq!(Range::new(2, 6).intersects(&Range::new(3, 7)), true);
    assert_eq!(Range::new(6, 10).intersects(&Range::new(2, 5)), false);
    assert_eq!(Range::new(2, 5).intersects(&Range::new(6, 9)), false);
}

fn main() {
    let input = BufReader::new(File::open("input").expect("input file to exist and be readable."));

    let mut containing_pairs = 0;
    let mut overlapping_pairs = 0;
    for line in input.lines() {
        let line = line.expect("to have been read correctly");
        let mut parser = TinyParser::new(line.trim());
        let first = Range::parser(&mut parser);
        parser.match_(',');
        let second = Range::parser(&mut parser);
        if first.size_pair(&second).containing() {
            containing_pairs += 1;
        }
        if first.intersects(&second) {
            overlapping_pairs += 1;
        }
    }
    
    println!("There are {} pairs where one element fully contains the other.", containing_pairs);
    println!("There are {} pairs where one element intersects the other.", overlapping_pairs);
}
