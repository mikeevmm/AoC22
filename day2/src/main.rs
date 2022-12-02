use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::thread;

macro_rules! own {
    (mut $x:ident) => {
        let mut $x = $x;
    };
    ($x:ident) => {
        let $x = $x;
    };
}

fn main() {
    let readin = BufReader::new(File::open("input").expect("the file to exist and be readable"));

    let (tx, rx) = mpsc::channel::<(u32, u32)>();
    let mut as_play_score = 0;
    let mut as_outcome_score = 0;

    thread::scope(|scope| {
        scope.spawn(|| {
            own!(rx);
            while let Ok((a, b)) = rx.recv() {
                as_play_score += a;
                as_outcome_score += b;
            }
        });

        for line in readin.lines() {
            let line = line.expect("every line in the file to be readable");
            let tx = tx.clone();
            scope.spawn(move || {
                let mut chars = line.chars();
                let opponent: u8 = match chars.next().expect("the line to have a first character") {
                    'A' => 0,
                    'B' => 1,
                    'C' => 2,
                    c => {
                        panic!("Unexpected character {} found in the first column!", c);
                    }
                };
                match chars.next() {
                    Some(' ') => {}
                    _ => {
                        panic!("Expected the line to have a space after the first column");
                    }
                }

                let second_col: u8 = match chars.next().expect("the line to have a third column") {
                    'X' => 0,
                    'Y' => 1,
                    'Z' => 2,
                    c => {
                        panic!("Unexpected character {} found in the third column!", c);
                    }
                };
                
                // The %s below represent a useful way to think of the problem (since it has a cyclic structure):
                //      If R = 0, P = 1, S = 2, notice that (x+1)%3 gives the throw to which x is weak.
                //      Likewise, (x+2)%3 gives the throw to which x is strong.

                let if_as_played: u32 = if second_col == ((opponent + 1) % 3) {
                    6 /* win */
                } else if opponent == ((second_col + 1) % 3) {
                    0 /* loss */
                } else {
                    3 /* tie */
                };

                let if_as_outcome = (opponent + 2 + second_col) % 3;

                tx.send((
                    if_as_played + (second_col as u32 + 1),
                    second_col as u32 * 3 + (if_as_outcome as u32 + 1),
                ))
                .expect("the sum channel to be listening");
            });
        }

        drop(tx);
    });

    println!(
        "Part 1 answer: {}\nPart 2 answer: {}",
        as_play_score, as_outcome_score
    );
}
