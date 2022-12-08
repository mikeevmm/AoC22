use itertools::Itertools;
use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
};

pub type Height = u8;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Node(pub usize, pub usize);

impl Node {
    pub fn offset(self, direction: &Direction, count: usize) -> Node {
        match direction {
            Direction::Up => Node(self.0 - count, self.1),
            Direction::Down => Node(self.0 + count, self.1),
            Direction::Left => Node(self.0, self.1 - count),
            Direction::Right => Node(self.0, self.1 + count),
        }
    }
}

fn part1(map: &Vec<Vec<Height>>) {
    let width = map[0].len();
    let height = map.len();
    let mut seen = BTreeSet::new();

    // Horizontal slices
    for slice in 0..width {
        seen.insert((slice, 0));
        seen.insert((slice, width - 1));
        let (mut left, mut right) = (0, width - 1);
        let (mut max_left, mut max_right) = (map[slice][left], map[slice][right]);
        for _ in 1..width {
            left += 1;
            right -= 1;
            let (left_height, right_height) = (map[slice][left], map[slice][right]);
            if left_height > max_left {
                max_left = left_height;
                seen.insert((slice, left));
            }
            if right_height > max_right {
                max_right = right_height;
                seen.insert((slice, right));
            }
        }
    }

    // Vertical slices
    for slice in 0..height {
        seen.insert((0, slice));
        seen.insert((height - 1, slice));
        let (mut top, mut bottom) = (0, height - 1);
        let (mut max_top, mut max_bottom) = (map[top][slice], map[bottom][slice]);
        for _ in 1..width {
            top += 1;
            bottom -= 1;
            let (top_height, bottom_height) = (map[top][slice], map[bottom][slice]);
            if top_height > max_top {
                max_top = top_height;
                seen.insert((top, slice));
            }
            if bottom_height > max_bottom {
                max_bottom = bottom_height;
                seen.insert((bottom, slice));
            }
        }
    }

    println!("A total of {} trees are visible.", seen.len());
}

type Visibility = u32;

fn part2(map: &Vec<Vec<Height>>) {
    let map_width = map[0].len();
    let map_height = map.len();

    let best = (1..(map_height - 1))
        .cartesian_product(1..(map_width - 1))
        .map(|(i, j)| Node(i, j))
        .map(|n| {
            let height = map[n.0][n.1];
            recursive_visibility(
                &n.offset(&Direction::Down, 1),
                &Direction::Down,
                height,
                map_width,
                map_height,
                map,
            ) * recursive_visibility(
                &n.offset(&Direction::Left, 1),
                &Direction::Left,
                height,
                map_width,
                map_height,
                map,
            ) * recursive_visibility(
                &n.offset(&Direction::Right, 1),
                &Direction::Right,
                height,
                map_width,
                map_height,
                map,
            ) * recursive_visibility(
                &n.offset(&Direction::Up, 1),
                &Direction::Up,
                height,
                map_width,
                map_height,
                map,
            )
        })
        .max()
        .expect("some answer");
    println!("The best scenic score is {}", best);
}

fn at_edge(node: &Node, direction: &Direction, map_width: usize, map_height: usize) -> bool {
    match direction {
        Direction::Up => node.0 == 0,
        Direction::Down => node.0 == map_height - 1,
        Direction::Left => node.1 == 0,
        Direction::Right => node.1 == map_width - 1,
    }
}

fn recursive_visibility(
    from: &Node,
    direction: &Direction,
    ref_height: Height,
    map_width: usize,
    map_height: usize,
    map: &Vec<Vec<Height>>,
) -> Visibility {
    if at_edge(from, direction, map_width, map_height) {
        return 1;
    }
    let height_here = map[from.0][from.1];
    let visibility = if height_here >= ref_height {
        1
    } else {
        let next = from.offset(direction, 1);
        1 + recursive_visibility(&next, direction, ref_height, map_width, map_height, map)
    };
    return visibility;
}

fn main() {
    let map: Vec<Vec<Height>> = {
        BufReader::new(File::open("input").expect("to exist and be readable"))
            .lines()
            .map(|l| {
                l.expect("line to be readable")
                    .chars()
                    .map(|c| {
                        c.to_digit(10)
                            .expect("every character to be parseable as a u8")
                            as Height
                    })
                    .collect()
            })
            .collect()
    };

    part1(&map);
    part2(&map);
}
