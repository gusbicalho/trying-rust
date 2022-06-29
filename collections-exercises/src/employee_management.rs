pub const NAME: &str = "Employee Management";

// Using a hash map and vectors, create a text interface to allow a user to add
// employee names to a department in a company. For example,
// “Add Sally to Engineering” or “Add Amir to Sales.” Then let the user retrieve
// a list of all people in a department or all people in the company by
// department, sorted alphabetically.

pub fn run() {
    println!("{}", NAME);
    let mut registry = registry::new();
    for line in prompt::prompts("> ") {
        use commands::Command;
        match line.parse() {
            Err(_) => {
                println!("Unknown command!");
            }
            Ok(command) => match command {
                Command::AddEmployee(ae) => {
                    registry.add_employee(&ae.name, &ae.department);
                }
                Command::Report(report) => {
                    for employee in registry.list_people(report.department.as_deref()) {
                        println!("{}", employee);
                    }
                }
            },
        };
    }
}

mod prompt {
    use std::io::{self, Write};
    use std::iter::Iterator;
    use super::util;

    pub fn prompt(prompt: &str) -> Option<String> {
        let mut buffer = String::new();
        {
            let mut stdout = io::stdout().lock();
            stdout.write_fmt(format_args!("{}", prompt)).ok()?;
            stdout.flush().ok()?;
        }
        io::stdin().read_line(&mut buffer).ok()?;
        util::trim_in_place(&mut buffer);
        if buffer.is_empty() {
            None
        } else {
            Some(buffer)
        }
    }

    pub fn prompts(prompt: &str) -> impl Iterator<Item = String> {
        StdPromptIter {
            prompt: prompt.to_string(),
        }
        .fuse()
    }

    pub struct StdPromptIter {
        prompt: String,
    }
    impl Iterator for StdPromptIter {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            prompt(&self.prompt)
        }
    }
}

mod registry {
    use super::util;
    use std::collections::HashMap;
    static EMPTY_VEC: Vec<String> = Vec::new();

    pub struct Registry {
        all_employees: Vec<String>,
        employees_by_department: HashMap<String, Vec<String>>,
    }
    pub fn new() -> Registry {
        Registry {
            all_employees: Vec::new(),
            employees_by_department: HashMap::new(),
        }
    }
    impl Registry {
        pub fn add_employee(&mut self, name: &str, department: &str) {
            util::insert_sorted_unique(&mut self.all_employees, name.to_string());
            util::insert_sorted_unique(
                self.employees_by_department
                    .entry(department.to_string())
                    .or_insert(vec![]),
                name.to_string(),
            );
        }
        pub fn list_people(&self, department: Option<&str>) -> std::slice::Iter<'_, String> {
            match department {
                None => self.all_employees.iter(),
                Some(department) => match self.employees_by_department.get(department) {
                    Some(employees) => employees.iter(),
                    None => EMPTY_VEC.iter(),
                },
            }
        }
    }
}

mod commands {
    // TODO detailed parse errors :shrug:
    pub struct ParseError {}

    use std::str::FromStr;
    pub enum Command {
        AddEmployee(AddEmployee),
        Report(Report),
    }

    impl FromStr for Command {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse()
                .map(Command::AddEmployee)
                .or_else(|_| s.parse().map(Command::Report))
        }
    }

    pub struct AddEmployee {
        pub name: String,
        pub department: String,
    }
    impl FromStr for AddEmployee {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut parts = s.split(' ');
            if parts.next().ok_or(ParseError {})? != "Add" {
                Err(ParseError {})?;
            };
            let name = parts.next().ok_or(ParseError {})?.to_string();
            if parts.next().ok_or(ParseError {})? != "to" {
                Err(ParseError {})?;
            };
            let department = parts.next().ok_or(ParseError {})?.trim().to_string();
            Ok(AddEmployee { name, department })
        }
    }

    pub struct Report {
        pub department: Option<String>,
    }
    impl FromStr for Report {
        type Err = ParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut parts = s.split(' ');
            if parts.next().ok_or(ParseError {})?.trim() != "List" {
                Err(ParseError {})?;
            };
            Ok(Report {
                department: parts.next().map(str::trim).map(String::from),
            })
        }
    }
}

mod util {
    use std::cmp::Ordering;
    pub fn insert_sorted_unique<T: Ord>(vec: &mut Vec<T>, new_s: T) {
        fn find_index<T: Ord>(vec: &[T], target: &T) -> Option<usize> {
            for (i, s) in vec.iter().enumerate() {
                match s.cmp(target) {
                    Ordering::Equal => return None,
                    Ordering::Greater => return Some(i),
                    Ordering::Less => continue,
                }
            }
            Some(vec.len())
        }
        if let Some(index) = find_index(vec, &new_s) {
            vec.insert(index, new_s);
        }
    }

    // from https://users.rust-lang.org/t/trim-string-in-place/15809/9
    pub fn trim_in_place(this: &mut String) {
        let trimmed: &str = this.trim();
        let trim_start = trimmed.as_ptr() as usize - this.as_ptr() as usize;
        let trim_len = trimmed.len();
        if trim_start != 0 {
            this.drain(..trim_start);
        }
        this.truncate(trim_len);
    }

    #[cfg(test)]
    mod trim_in_place_tests {
        use super::*;

        #[test]
        fn nothing_to_trim() {
            check_trimmed("such content wow", "such content wow");
        }
        #[test]
        fn trim_left() {
            check_trimmed(" \n\tsuch content wow", "such content wow");
        }
        #[test]
        fn trim_right() {
            check_trimmed("such content wow \r\n \t", "such content wow");
        }
        #[test]
        fn trim_both() {
            check_trimmed(" \n\tsuch content wow \r\n \t", "such content wow");
        }

        fn check_trimmed(original: &str, trimmed: &str) {
            let mut s = original.to_string();
            trim_in_place(&mut s);
            assert_eq!(s, trimmed);
        }
    }
}
