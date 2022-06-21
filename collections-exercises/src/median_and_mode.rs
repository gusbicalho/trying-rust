pub const NAME: &str = "Median and Mode";

// Given a list of integers, use a vector and return
// the median (when sorted, the value in the middle position) and
// the mode (the value that occurs most often; a hash map will be helpful here)
// of the list.

use std::io;
use std::ops::{AddAssign};

pub fn run() {
    println!("Input numbers, one per line:");
    let mut numbers: Vec<i32> = util::repeatedly(|| util::read_line_as(&mut io::stdin().lock()));
    numbers.sort_unstable();
    println!("{:?}", numbers);
    println!("Median: {:?}", middle(&numbers));
    println!("Mode: {:?}", mode(&numbers));
}

fn middle(numbers: &[i32]) -> Option<i32> {
    numbers.get(numbers.len() / 2).copied()
}

fn mode(numbers: &Vec<i32>) -> Option<i32> {
    use std::collections::HashMap;
    let mut frequencies: HashMap<i32, u32> = HashMap::new();
    for n in numbers {
        frequencies.entry(*n).or_insert(0).add_assign(1);
    }
    let mut mode = (None, 0);
    for (n, occurences) in frequencies {
        if occurences > mode.1 {
            mode = (Some(n), occurences);
        }
    }
    mode.0
}

mod util {
    use std::io::BufRead;
    use std::str::FromStr;
    pub fn read_line_as<R: FromStr, In: BufRead>(read_in: &mut In) -> Option<R> {
        let mut buf = String::new();
        read_in.read_line(&mut buf).ok()?;
        buf.trim().parse().ok()
    }

    pub fn repeatedly<R>(action: fn() -> Option<R>) -> Vec<R> {
        let mut results: Vec<R> = vec![];
        loop {
            match action() {
                None => break,
                Some(result) => results.push(result),
            }
        }
        results
    }
}
