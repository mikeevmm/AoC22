use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
struct Coordinate {
    row: usize,
    col: usize,
}

#[derive(Debug)]
struct HeightMap {
    inner: Vec<u8>,
    width: usize,
    height: usize,
    start: Coordinate,
    goal: Coordinate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Neighbours<'map, 'center> {
    map: &'map HeightMap,
    center: &'center Coordinate,
    next_dir: Direction,
    exhausted: bool,
}

impl From<(usize, usize)> for Coordinate {
    fn from((row, col): (usize, usize)) -> Self {
        Coordinate { row, col }
    }
}

impl Coordinate {
    /// Hamming distance.
    fn distance_to(&self, other: &Coordinate) -> usize {
        other.col.abs_diff(self.col) + other.row.abs_diff(self.row)
    }
}

impl HeightMap {
    fn get_coordinate(&self, coord: &Coordinate) -> u8 {
        self.inner[coord.row * self.width + coord.col]
    }

    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let input = File::open(path)?;
        let mut line_width = None;
        let mut heights = vec![];
        let mut rows = 0;
        let mut start = None;
        let mut goal = None;
        let mut i = 0; // `enumerate` would be messed up by '\r's.
        for char_ in input.bytes() {
            let char_ = char_? as char;
            if char_ == '\r' {
                continue;
            }
            let col = if let Some(lw) = line_width {
                (i - (rows % lw)) % lw
            } else {
                i
            };
            match char_ {
                '\n' => {
                    if line_width.is_none() {
                        line_width = Some(i);
                    } else {
                        let line_width = line_width.unwrap();
                        if (i % line_width) != (rows % line_width) {
                            return Err("Inconsistent line widths".into());
                        }
                    }
                    rows += 1;
                }
                'S' => {
                    if start.is_some() {
                        return Err("Multiple starts found".into());
                    }
                    start = Some((rows, col).into());
                    heights.push('a' as u8); // Essentially a dummy value
                }
                'E' => {
                    if goal.is_some() {
                        return Err("Multiple goals found".into());
                    }
                    goal = Some((rows, col).into());
                    heights.push('i' as u8); // FIXME
                }
                c => heights.push(c as u8),
            }
            i += 1;
        }
        let start = start.ok_or("No start found".to_string())?;
        let goal = goal.ok_or("No goal found".to_string())?;
        let line_width = line_width.unwrap_or_else(|| heights.len());

        Ok(HeightMap {
            inner: heights,
            width: line_width,
            height: rows + 1,
            start,
            goal,
        })
    }

    fn neighbours<'n>(&'n self, to: &'n Coordinate) -> impl Iterator<Item = Coordinate> + 'n {
        let center_height = self.get_coordinate(&to);
        Neighbours::to(&self, to).filter(move |x| {
            let other_height = self.get_coordinate(x);
            other_height < center_height || other_height - center_height <= 1
        })
    }

    fn navigate(&self) -> Option<usize> {
        // A Star; how fitting!

        let mut cost = HashMap::<Coordinate, usize>::new();
        let mut parent = HashMap::<Coordinate, Coordinate>::new();
        let mut heuristic = BTreeSet::<(usize, Coordinate)>::new();

        cost.insert(self.start, 0);
        heuristic.insert((self.start.distance_to(&self.goal), self.start));

        let goal = loop {
            let first = heuristic.pop_first();
            if first.is_none() {
                break None;
            }
            let (_heuristic_score, current) = first.unwrap();
            if current == self.goal {
                break Some(current);
            }

            let current_score = *cost.get(&current).unwrap();

            for neighbour in self.neighbours(&current) {
                let tentative_cost = current_score + 1;
                let neighbour_cost = *cost.get(&neighbour).unwrap_or(&usize::MAX);
                if tentative_cost < neighbour_cost {
                    let old_neighbour_heuristic =
                        neighbour_cost.saturating_add(neighbour.distance_to(&self.goal));
                    heuristic.remove(&(old_neighbour_heuristic, neighbour));
                    let neighbour_heuristic = tentative_cost + neighbour.distance_to(&self.goal);
                    parent.insert(neighbour, current);
                    cost.insert(neighbour, tentative_cost);
                    heuristic.insert((neighbour_heuristic, neighbour));
                }
            }
        };

        if goal.is_none() {
            return None;
        }

        // Backtrack the goal
        let mut backtrack = goal.unwrap();
        let mut count = 0;
        loop {
            match parent.get(&backtrack) {
                Some(p) => {
                    count += 1;
                    backtrack = *p;
                }
                None => {
                    if backtrack != self.start {
                        panic!("Something went wrong")
                    } else {
                        break;
                    }
                }
            }
        }

        Some(count)
    }
}

impl Direction {
    fn next(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    fn first() -> Self {
        Direction::Up
    }

    fn last() -> Self {
        Direction::Left
    }
}

impl<'this, 'coord> Neighbours<'this, 'coord> {
    fn to(map: &'this HeightMap, center: &'coord Coordinate) -> Self {
        Neighbours {
            map,
            center,
            next_dir: Direction::first(),
            exhausted: false,
        }
    }
}

impl<'this, 'coord> Iterator for Neighbours<'this, 'coord> {
    type Item = Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }
        let dir = self.next_dir.clone();
        if dir == Direction::last() {
            self.exhausted = true;
        }
        self.next_dir = self.next_dir.next();
        let to_yield = match dir {
            Direction::Up => {
                if self.center.row == 0 {
                    return self.next();
                }
                (self.center.row - 1, self.center.col)
            }
            Direction::Down => {
                if self.center.row == self.map.height - 1 {
                    return self.next();
                }
                (self.center.row + 1, self.center.col)
            }
            Direction::Left => {
                if self.center.col == 0 {
                    return self.next();
                }
                (self.center.row, self.center.col - 1)
            }
            Direction::Right => {
                if self.center.col == self.map.width - 1 {
                    return self.next();
                }
                (self.center.row, self.center.col + 1)
            }
        };
        Some(to_yield.into())
    }
}

#[test]
fn parse() {
    println!("{:?}", HeightMap::from_file("input"))
}

#[test]
fn neighbours() {
    let map = HeightMap::from_file("input").unwrap();
    for neighbour in map.neighbours(&(1, 3).into()) {
        println!("{:?}", neighbour);
    }
}

#[test]
fn exampleinput() {
    let map = HeightMap::from_file("exampleinput").unwrap();
    assert_eq!(map.navigate().unwrap(), 31);
}

#[test]
fn exampleinput2() {
    let map = HeightMap::from_file("exampleinput2").unwrap();
    map.navigate();
}

fn main() {
    // Part 1
    let map = HeightMap::from_file("input").unwrap();
    println!("You are {} steps away from the goal.", map.navigate().unwrap());

    // Part 2
    let possible_starts: Vec<usize> = map
        .inner
        .iter()
        .enumerate()
        .filter(|(i, &x)| x == 'a' as u8)
        .map(|(i, _)| i)
        .collect();
    let possible_starts: Vec<Coordinate> = possible_starts
        .into_iter()
        .map(|i| (i / map.width, i % map.width).into())
        .collect();

    let mut map = map;
    let mut best_start = None;
    for start in possible_starts {
        map.start = start;
        if let Some(steps) = map.navigate() {
            best_start = Some(match best_start {
                Some((best_steps, best_start)) => {
                    if steps < best_steps {
                        (steps, start)
                    } else {
                        (best_steps, best_start)
                    }
                }
                None => (steps, start),
            });
        }
    }
    println!("Best start takes {} steps.", best_start.unwrap().0)
}
