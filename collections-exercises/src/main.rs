mod median_and_mode;
mod pig_latin;
mod employee_management;

use std::io;
fn main() {
    let exercises = [
        (median_and_mode::NAME, median_and_mode::run as fn()),
        (pig_latin::NAME, pig_latin::run as fn()),
        (employee_management::NAME, employee_management::run as fn()),
    ];
    let mut buf = String::new();
    loop {
        println!("Pick one!");
        for (i, (name, _)) in exercises.iter().enumerate() {
            println!("{} - {}", i, name);
        }
        buf.clear();
        io::stdin().read_line(&mut buf).expect("Failed to read stdin");
        let choice: Option<usize> = buf.trim().parse().ok();
        match choice.and_then(|i| { exercises.get(i) }) {
            None => continue,
            Some((name, run_fn)) => {
                println!("=== {} ===", name);
                run_fn();
                break;
            }
        }
    }
}
