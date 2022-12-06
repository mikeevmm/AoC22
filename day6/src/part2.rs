use std::{
    fs::File,
    io::{BufReader, Read}, path::Path,
};

use crate::shift::CircularBuffer;

fn open_file_as_char_iter(file: impl AsRef<Path>) -> impl Iterator<Item = char> {
    BufReader::new(File::open(file).expect("input to exist and be readable"))
        .bytes()
        .map(|b| b.expect("every byte to be readable") as char)
}

pub fn main() {
    let mut input = open_file_as_char_iter("input");
    let mut history = CircularBuffer::<14, char>::from([' '; 14]);

    // Take the first 14 characters to fill the buffer.
    for c in input.by_ref().take(14) {
        history.push(c);
    }
    
    // Then, consume the rest of the input.
    let mut count = 14;
    'outer: loop {
        for i in (1..=13).rev() {
            let ith = *history.ith_oldest(i);
            for j in (0..=i-1).rev() {
                let jth = *history.ith_oldest(j);
                if ith == jth {
                    // Shift the history until the ith element is gone.
                    count +=  i;
                    for c in input.by_ref().take(i) {
                        history.push(c);
                    }
                    continue 'outer;
                }
            }
        }
        break;
    }
    
    println!("Ate {} characters consumed before finding a start-of-message sequence.", count);
}
