#![feature(iter_array_chunks)]

use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(PartialEq, Eq)]
struct ItemType {
    inner: u8,
}

impl ItemType {
    fn numeric(&self) -> u8 {
        let ascii = self.inner;
        if ascii > 96 {
            ascii - 96
        } else {
            ascii - 64 + 26
        }
    }
}

impl From<&char> for ItemType {
    fn from(c: &char) -> Self {
        ItemType { inner: *c as u8 }
    }
}

impl From<&u8> for ItemType {
    fn from(byte: &u8) -> Self {
        ItemType { inner: *byte }
    }
}

impl From<u8> for ItemType {
    fn from(byte: u8) -> Self {
        ItemType { inner: byte }
    }
}

impl PartialOrd for ItemType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for ItemType {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.numeric().cmp(&other.numeric())
    }
}

struct Rucksack<'items> {
    items: &'items [u8],
    /// Sorted by ASCII code, not ItemType::numeric().
    left_sorted: Vec<usize>,
    /// Sorted by ASCII code, not ItemType::numeric().
    right_sorted: Vec<usize>,
}

impl<'i> Rucksack<'i> {
    fn new(items: &'i str) -> Self {
        let items = items.as_bytes();

        // Sorting (with deduplication) is O(n log(n)), and then finding a duplicate is just O(n)
        // making the whole thing Õ(n). Comparing each item in the first half to each item in the
        // second half would be O(n²).
        // (We pay O(n) of memory for it.)

        let mut left_sorted: Vec<usize> = vec![];
        let mut right_sorted: Vec<usize> = vec![];

        let section_size = items.len() / 2;
        // This iterator will iterate over pairs of (index, &elem) for each section.
        let left_and_right_iterator = (0..section_size)
            .zip(section_size..)
            .map(|(li, ri)| ((li, &items[li]), (ri, &items[ri])));

        for ((li, lc), (ri, rc)) in left_and_right_iterator {
            // Insert sorted for the left section
            match left_sorted.binary_search_by(|&other| items[other].cmp(lc)) {
                Ok(_) => { /* De-duplicate */ }
                Err(insert_at) => {
                    left_sorted.insert(insert_at, li);
                }
            }

            // Insert sorted for the right section
            match right_sorted.binary_search_by(|&other| items[other].cmp(rc)) {
                Ok(_) => { /* De-duplicate */ }
                Err(insert_at) => {
                    right_sorted.insert(insert_at, ri);
                }
            }
        }

        Rucksack {
            items,
            left_sorted,
            right_sorted,
        }
    }

    fn duplicate<'this>(self: &'this Self) -> ItemType {
        // Lockstep
        let mut left_ptr = 0;
        let mut right_ptr = 0;
        let input_size = self.items.len() / 2;
        while left_ptr < input_size && right_ptr < input_size {
            let left = self.items[self.left_sorted[left_ptr]];
            let right = self.items[self.right_sorted[right_ptr]];

            match left.cmp(&right) {
                std::cmp::Ordering::Equal => {
                    return left.into();
                }
                std::cmp::Ordering::Less => {
                    left_ptr += 1;
                }
                std::cmp::Ordering::Greater => {
                    right_ptr += 1;
                }
            }
        }
        panic!("No duplicates found, input is malformed")
    }
}

struct Group {
    lines: [Vec<u8>; 3],
}

impl From<[String; 3]> for Group {
    fn from(lines: [String; 3]) -> Self {
        Group {
            // FIXME: There's some unsafe efficiency trickery to be done here.
            lines: [
                lines[0].bytes().collect(),
                lines[1].bytes().collect(),
                lines[2].bytes().collect(),
            ],
        }
    }
}

impl Group {
    fn common(&self) -> ItemType {
        // Same logic as in the `Rucksack`, but for groups of three now
        // (and each line being its own entry, instead of two).
        let mut insert_sorted = [vec![], vec![], vec![]];

        let mut insert_sort_deduplicate = |group: usize, index: usize, c: u8| {
            let insert_sorted = &mut insert_sorted[group];
            let line: &[u8] = &self.lines[group];
            match insert_sorted.binary_search_by(|&other: &usize| line[other].cmp(&c)) {
                Ok(_) => { /* De-duplicate */ }
                Err(insert_at) => {
                    insert_sorted.insert(insert_at, index);
                }
            }
        };

        // It's not gorgeous, but probably the way to write this with least duplication!
        for (i1, c1) in self.lines[0].iter().enumerate() {
            insert_sort_deduplicate(0, i1, *c1);
        }
        for (i2, c2) in self.lines[1].iter().enumerate() {
            insert_sort_deduplicate(1, i2, *c2);
        }
        for (i3, c3) in self.lines[2].iter().enumerate() {
            insert_sort_deduplicate(2, i3, *c3);
        }

        // Same lockstep algorithm as before
        // The difference is we move forward the smallest entry (/entries)
        let mut ptr = [0, 0, 0];
        macro_rules! in_bounds {
            ($group: literal) => {{
                ptr[$group] < insert_sorted[$group].len()
            }};
        }
        while in_bounds!(0) && in_bounds!(1) && in_bounds!(2) {
            let first: u8 = self.lines[0][insert_sorted[0][ptr[0]]];
            let second: u8 = self.lines[1][insert_sorted[1][ptr[1]]];
            let third: u8 = self.lines[2][insert_sorted[2][ptr[2]]];
            if first == second && second == third {
                return first.into();
            }

            let min = std::cmp::min(std::cmp::min(first, second), third);
            if first == min && ptr[0] < insert_sorted[0].len() {
                ptr[0] += 1;
            }
            if second == min && ptr[1] < insert_sorted[1].len() {
                ptr[1] += 1;
            }
            if third == min && ptr[2] < insert_sorted[2].len() {
                ptr[2] += 1;
            }
        }
        panic!("No common entries found, input is malformed")
    }
}

fn part1() {
    let input = BufReader::new(File::open("input").expect("input file to exist and be readable"));

    let mut duplicates_sum: u32 = 0;
    for line in input.lines() {
        let line = line.expect("every line in the file to be readable");
        let rucksack = Rucksack::new(&line);
        duplicates_sum += rucksack.duplicate().numeric() as u32;
    }

    println!("Sum of duplicate values: {}", duplicates_sum);
}

fn part2() {
    let input = BufReader::new(File::open("input").expect("input file to exist and be readable"));

    let mut common_sum = 0;
    for [a, b, c] in input.lines().into_iter().array_chunks() {
        let (a, b, c) = (a.unwrap(), b.unwrap(), c.unwrap());
        common_sum += Group::from([a, b, c]).common().numeric() as u32;
    }

    println!("Sum of common values: {}", common_sum);
}

fn main() {
    part1();
    part2();
}
