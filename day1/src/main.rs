use std::fs::File;
use std::io::{BufRead, BufReader};
use std::arch::asm;

struct TopThree {
    values: [u32; 3],
}

impl TopThree {
    fn new() -> Self {
        TopThree {
            values: [0; 3],
        }
    }

    fn try_insert(&mut self, value: u32) {
        // It's obviously a little silly to do this in asm, but the problem is so constrained that
        // I felt like I could make it go quite fast.
        // 
        // ... Does this count as doing AoC in assembly, or...?
        unsafe {
            asm!("mov cl, 0",
                 //"0:", // ---> Compare eax with [rdi]
                 "cmp eax, [rdi]",
                 "jl 1f",
                 "add cl, 1", // Set the LSb of cl to signal we should put eax into [rdi]
                 "jmp 10f",
                 "1:", // ---> Compare eax with rdi[1]
                 "cmp eax, [rdi+4]",
                 "jl 2f",
                 "add cl, 2", // Set the second LSb of cl to signal we should put eax into rdi[1]
                 "jmp 10f",
                 "2:", // ---> Compare eax with rdi[2]
                 "cmp eax, [rdi+8]",
                 "jle 12f",
                 "mov [rdi+8], eax",
                 "jmp 12f",
                 "add cl, 1",
                 "10:", // ---> Shift rdi[1] to rdi[2]
                 "mov edx, [rdi+4]",
                 "mov [rdi+8], edx",
                 "test cl, 2", // If the second bit of cl is set, put eax into rdi[1]
                 "jz 11f",
                 "mov [rdi+4], eax",
                 "jmp 12f",
                 "11:", // ---> Shift rdi[0] to rdi[1]
                 "mov edx, [rdi]",
                 "mov [rdi+4], edx",
                 "test cl, 1",
                 "jz 12f", // If the first bit of cl is set, put eax into [rdi]
                 "mov [rdi], eax",
                 "12:", // ---> End
                in("rdi") self.values.as_mut_ptr(), in("eax") value, out("cl") _, out("edx") _)
        }
    }

    fn most(&self) -> &u32 {
        &self.values[0]
    }

    fn total(&self) -> u32 {
        self.values.iter().sum()
    }
}

fn main() {
    let readin =
        BufReader::new(File::open("input").expect("the input file to exist and be readable"));

    let mut top_three = TopThree::new();
    let mut acc = 0;

    for line in readin.lines() {
        let line = line.unwrap();
        let trimmed = line.trim();

        if trimmed.len() == 0 {
            // At the end of a block.
            top_three.try_insert(acc);
            acc = 0;
            continue;
        }

        let parsed = str::parse::<u32>(trimmed).unwrap();
        acc += parsed;
    }
    top_three.try_insert(acc);

    println!("Most calories: {}\nTop three total: {}", top_three.most(), top_three.total());
}
