use std::io::BufRead;
use std::{fs::File, io::BufReader};

mod parse;
mod fs;

#[derive(Debug)]
pub enum Entry {
    User(Command),
    Directory(String),
    File(u64, String),
}

#[derive(Debug)]
pub enum Command {
    Cd(String),
    Ls,
}

fn main() {
    let input = BufReader::new(File::open("input").expect("input file to exist and be readable"));

    let mut fs = fs::FileSystem::new(70_000_000);

    // Enumerate the file system
    for line in input.lines() {
        let line = line.expect("to be able to read every line");
        let entry = parse::entry(&line);
        match entry {
            Entry::User(Command::Cd(dir)) => fs.cd(dir),
            Entry::User(Command::Ls) => { /* File listings will follow */ }
            Entry::Directory(_dir) => { /* Dir is only important if we cd into it */ }
            Entry::File(size, name) => fs.put_file(name, size),
        }
    }
    
    let fs = fs.finalize();
    
    // Search recursively for directories with size less than a limit.
    println!("The sum of the sizes of the directories with at most 100000 is {}.", fs.total_size_below(100_000));
        
    // Search recursively for a directory to delete
    println!("The system is occupying {}.", fs.occupied_size());
    println!("To get enough space, we delete a directory of size {}.", fs.delete_to_free(30_000_000));
}
