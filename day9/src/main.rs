use owned_chars::OwnedCharsExt;
use std::{
    collections::BTreeSet,
    fmt::{write, Debug},
    fs::File,
    io::{BufRead, BufReader},
    ops::{Add, Sub},
};

use crate::part1::TwoKnotRope;

mod part1;

#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

type Count = u16;

#[derive(Clone, Copy)]
pub struct Movement {
    direction: Direction,
    count: Count,
}

struct MovementSeq {
    inner: std::io::Lines<BufReader<File>>,
}

type Coordinate = isize;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: Coordinate,
    y: Coordinate,
}

struct Delta {
    inner: Point,
}

struct Rope<const N: usize> {
    knots: [Point; N],
    visited: BTreeSet<Point>,
}

type GridSize = Point;
struct Grid<'r, const N: usize>(&'r Rope<N>, GridSize, Point);

impl MovementSeq {
    fn open(from: impl AsRef<std::path::Path>) -> Self {
        MovementSeq {
            inner: BufReader::new(File::open(from).expect("input to be readable")).lines(),
        }
    }
}

impl Iterator for MovementSeq {
    type Item = Movement;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.inner.next()?.expect("line to be readable");
        let mut chars = line.into_chars();
        let direction = match chars.next().expect("a first character (the direction)") {
            'U' => Direction::Up,
            'D' => Direction::Down,
            'L' => Direction::Left,
            'R' => Direction::Right,
            c => panic!("Unexpected character {} given as direction!", c),
        };
        match chars.next() {
            Some(' ') => {}
            _ => {
                panic!("Expected second character to be a space");
            }
        }
        let count: String = chars.take_while(|c| c.is_numeric()).collect();
        let count: Count =
            str::parse::<Count>(&count).expect("to be able to parse the count as a number");
        Some(Movement { direction, count })
    }
}

impl Point {
    fn zero() -> Self {
        Point { x: 0, y: 0 }
    }
}

impl From<(Coordinate, Coordinate)> for Point {
    fn from((x, y): (Coordinate, Coordinate)) -> Self {
        Point { x, y }
    }
}

impl Add<&Direction> for Point {
    type Output = Self;

    fn add(self, rhs: &Direction) -> Self::Output {
        let mut output = self.clone();
        match rhs {
            Direction::Up => output.y += 1,
            Direction::Down => output.y -= 1,
            Direction::Left => output.x -= 1,
            Direction::Right => output.x += 1,
        }
        output
    }
}

impl Add<&Delta> for Point {
    type Output = Self;

    fn add(self, rhs: &Delta) -> Self::Output {
        Point {
            x: self.x + rhs.inner.x,
            y: self.y + rhs.inner.y,
        }
    }
}

impl Sub for Point {
    type Output = Delta;

    fn sub(self, rhs: Self) -> Self::Output {
        Delta {
            inner: Point {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
            },
        }
    }
}

impl Delta {
    fn touching(&self) -> bool {
        (self.inner.x == 0 && self.inner.y.abs() == 1)
            || (self.inner.x.abs() == 1 && self.inner.y == 0)
            || (self.inner.x == 0 && self.inner.y == 0)
            || (self.inner.x.abs() == 1 && self.inner.y.abs() == 1)
    }

    fn normalized(self) -> Self {
        let inner = Point {
            x: self.inner.x.signum(),
            y: self.inner.y.signum(),
        };
        Delta { inner }
    }

    fn as_direction(self) -> Direction {
        if self.inner.x != 0 && self.inner.y != 0 {
            panic!("Tried to normalize a diagonal Delta");
        }
        if self.inner.y == 0 {
            if self.inner.x > 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else {
            if self.inner.y > 0 {
                Direction::Up
            } else {
                Direction::Down
            }
        }
    }
}

impl<const N: usize> Rope<N> {
    fn new() -> Self {
        let knots = [Point::zero(); N];
        let mut visited = BTreeSet::new();
        visited.insert(knots.last().unwrap().clone());
        Rope {
            knots,
            visited,
        }
    }

    fn move_head(&mut self, movement: Movement) {
        for _ in 0..movement.count {
            // Move the head.
            self.knots[0] = self.knots[0] + &movement.direction;

            for knot_index in 1..self.knots.len() {
                let knot = &self.knots[knot_index];
                let parent = &self.knots[knot_index - 1];
                let knot_to_parent = *parent - *knot;

                // If this knot does not need to move, the rest of the knots do not need to move.
                if knot_to_parent.touching() {
                    break;
                }

                // Now, moving this knot to the previous position of the parent no longer works.
                // Instead I'll just follow the rules outlined in the problem.
                if knot.x == parent.x || knot.y == parent.y {
                    // Same row or column; move knot in the direction of the delta.
                    self.knots[knot_index] =
                        self.knots[knot_index] + &knot_to_parent.as_direction();
                } else {
                    // Delta must be diagonal.
                    self.knots[knot_index] = self.knots[knot_index] + &knot_to_parent.normalized();
                }
                
                if knot_index == self.knots.len() - 1 {
                    self.visited.insert(self.knots.last().unwrap().clone());
                }
            }
        }
    }

    fn count_tail_visited(&self) -> usize {
        self.visited.len()
    }
}

impl Debug for Movement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            match self.direction {
                Direction::Up => "U",
                Direction::Down => "D",
                Direction::Left => "L",
                Direction::Right => "R",
            },
            self.count
        )
    }
}

impl<'r, const N: usize> Debug for Grid<'r, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in (0..self.1.y).rev() {
            let y = y - self.2.y;
            for x in 0..self.1.x {
                let x = x - self.2.x;
                let knot = self
                    .0
                    .knots
                    .iter()
                    .enumerate()
                    .find(|(_, k)| k.x == x && k.y == y);
                if let Some((i, _)) = knot {
                    write!(f, "{}", i)?;
                } else {
                    if self.0.visited.contains(&(x, y).into()) {
                        write!(f, "#")?;
                    } else {
                        write!(f, ".")?;
                    }
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}

#[test]
fn example1() {
    let movement_sequence = MovementSeq::open("exampleinput1");
    let mut rope = TwoKnotRope::new();

    println!("{:?}", rope);
    for head_movement in movement_sequence {
        println!("{:?}", head_movement);
        rope.move_head(head_movement);
    }

    assert_eq!(rope.count_tail_visited(), 13)
}

#[test]
fn example2() {
    let movement_sequence = MovementSeq::open("exampleinput2");
    let mut rope = Rope::<10>::new();

    for head_movement in movement_sequence {
        rope.move_head(head_movement);
    }

    assert_eq!(rope.count_tail_visited(), 36)
}

fn main() {
    let movement_sequence = MovementSeq::open("input");
    let mut two_knot = TwoKnotRope::new();
    let mut ten_knot = Rope::<10>::new();

    for head_movement in movement_sequence {
        two_knot.move_head(head_movement);
        ten_knot.move_head(head_movement);
    }

    println!(
        "1: The tail visited {} positions.",
        two_knot.count_tail_visited()
    );
    println!(
        "2: The tail visited {} positions.",
        ten_knot.count_tail_visited()
    );
}
