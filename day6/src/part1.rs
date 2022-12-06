use std::{io::{BufReader, Read}, fs::File};
use crate::shift::CircularBuffer;

struct Stream {
    history: CircularBuffer<3, char>,
    count: u32,
}

impl Stream {
    fn new(first_three_chars: [u8; 3]) -> Self {
        Stream {
            history: [
                first_three_chars[2] as char,
                first_three_chars[1] as char,
                first_three_chars[0] as char,
            ]
            .into(),
            count: 3,
        }
    }

    fn eat(&mut self, c: char) -> bool {
        self.count += 1;
        let pushed_out = *self.history.oldest();
        self.history.push(c);
        if pushed_out == c {
            // The character we're pushing out matches the character coming in.
            // Keep looking.
            return false;
        }
        // Else:
        // The character we're pushing out doesn't match the character coming in.
        // Was it different to the remaining characters, and are they different amongst
        // themselves?
        let mut all_different = true;
        for i in 0..3 {
            let ith_element = *self.history.ith_newest(i);
            if ith_element == pushed_out {
                all_different = false;
                break;
            }
            for j in (i+1)..3 {
                let jth_element = *self.history.ith_newest(j);
                if ith_element == jth_element {
                    all_different = false;
                    break;
                }
            }
        }
        all_different
    }
}

pub fn main() {
    let mut input = BufReader::new(File::open("input").expect("input to exist and be readable"));
    let mut first_three_chars = [0_u8; 3];
    input.read_exact(&mut first_three_chars).expect("to be able to read the first three characters");
    let mut stream = Stream::new(first_three_chars);
    
    for c in input.bytes() {
        let c = c.expect("every byte to be readable") as char;
        let halt = stream.eat(c);
        if halt {
            break;
        }
    }
    
    println!("Ate {} characters before getting a halt sequence.", stream.count);
}
