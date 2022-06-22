pub const NAME: &str = "Pig Latin";

// Convert strings to pig latin.
// The first consonant of each word is moved to the end of the word and
// “ay” is added, so “first” becomes “irst-fay.”
// Words that start with a vowel have “hay” added to the end instead (“apple”
// becomes “apple-hay”). Keep in mind the details about UTF-8 encoding!

use std::io;

pub fn run() {
    println!("Input a message:");
    let input = util::with_write_buffer_(|buf| {
        io::stdin().read_line(buf).expect("Stdin failure");
    });
    let result = util::with_write_buffer_(|buf| {
        util::transform_words(to_pig_latin_word, &input, buf);
    });
    println!("{}", result);
}

fn to_pig_latin_word(word: &str, output: &mut String) {
    let mut chars = word.chars();
    if let Some(first_char) = chars.next() {
        if is_vowel(first_char) {
            output.push(first_char);
            output.push_str(chars.as_str());
            output.push_str("-hay");
        } else {
            output.push_str(chars.as_str());
            output.push('-');
            output.push(first_char);
            output.push_str("ay");
        }
    }
}

fn is_vowel(c: char) -> bool {
    "AEIOUaeiou".contains(c)
}

mod util {
    pub fn with_write_buffer<F, R>(action: F) -> (R, String)
    where
        F: FnOnce(&mut String) -> R,
    {
        let mut output = String::new();
        let result = action(&mut output);
        (result, output)
    }

    pub fn with_write_buffer_<F>(action: F) -> String
    where
        F: FnOnce(&mut String),
    {
        with_write_buffer(action).1
    }

    pub fn transform_words(
        transform: fn(&str, &mut String) -> (),
        input: &str,
        output: &mut String,
    ) {
        let mut word = String::new();
        for c in input.chars() {
            if c.is_alphabetic() {
                word.push(c)
            } else {
                if !word.is_empty() {
                    transform(&word, output);
                    word.clear();
                }
                output.push(c);
            }
        }
    }
}
