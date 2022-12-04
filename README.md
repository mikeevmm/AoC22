# Advent of Code 2022

This repository contains my solutions to [Advent of Code 2022][aoc22].

I'm solving the problems with Rust. I've tried to keep things clean, though I'm not shying away from the more cursed stuff (day 1 uses assembly; how worse could it get).

As usual, these done are to my best knowledge at the time of writing, but you shouldn't consider anything here as objectively correct before checking.

## Notes

**Day 3**: I've realized that by using a `HashSet` I could have attained complexity O(n) rather than Õ(n). I'm not entirely sure why I did things the way I did, which was: 1) key sorting each "rucksack" by the ASCII code, and then 2) move pointers over these sorting keys such that the one pointing to the smallest entry moves forward each iteration, until one entry matches.

[aoc22]: https://adventofcode.com/2022/
