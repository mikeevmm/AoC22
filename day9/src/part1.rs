use crate::{Movement, Point};
use core::fmt::Debug;
use std::collections::BTreeSet;

pub struct TwoKnotRope {
    head_position: Point,
    tail_position: Point,
    visited: BTreeSet<Point>,
}

impl TwoKnotRope {
    pub fn new() -> Self {
        let mut visited = BTreeSet::new();
        visited.insert(Point::zero());
        TwoKnotRope {
            head_position: Point::zero(),
            tail_position: Point::zero(),
            visited,
        }
    }

    pub fn move_head(&mut self, movement: Movement) {
        for _ in 0..movement.count {
            let old_head_position = self.head_position.clone();
            self.head_position = self.head_position + &movement.direction;

            // Now, we just need to check whether the tail position needs to move, taking into account
            // the new head position.
            let tail_moved = Self::adjust_tail(
                &self.head_position,
                &mut self.tail_position,
                old_head_position,
            );

            // Mark the position that the tail is now on.
            if tail_moved {
                self.visited.insert(self.tail_position.clone());
            }
        }
    }

    fn adjust_tail(head: &Point, tail: &mut Point, old_head_position: Point) -> bool {
        let tail_to_head = *head - *tail;
        if tail_to_head.touching() {
            return false;
        }
        *tail = old_head_position;
        true
    }

    pub fn count_tail_visited(&self) -> usize {
        self.visited.len()
    }
}

impl Debug for TwoKnotRope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in (0..5).rev() {
            for j in 0..6 {
                if self.head_position.x == j && self.head_position.y == i {
                    write!(f, "H")?;
                } else if self.tail_position.x == j && self.tail_position.y == i {
                    write!(f, "T")?;
                } else if self.visited.contains(&(j, i).into()) {
                    write!(f, "#")?;
                } else if i == 0 && j == 0 {
                    write!(f, "s")?;
                } else {
                    write!(f, ".")?;
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}