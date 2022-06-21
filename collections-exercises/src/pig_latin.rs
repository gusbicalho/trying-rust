pub const NAME: &str = "Pig Latin";

// Convert strings to pig latin.
// The first consonant of each word is moved to the end of the word and
// “ay” is added, so “first” becomes “irst-fay.”
// Words that start with a vowel have “hay” added to the end instead (“apple”
// becomes “apple-hay”). Keep in mind the details about UTF-8 encoding!

use std::io;

pub fn run() {
    println!("Input a message:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Stdin failure");
    let input = input;
    let mut output = String::new();
    {
        let mut word = String::new();
        for c in input.chars() {
            if c.is_alphabetic() {
                word.push(c)
            } else {
                if !word.is_empty() {
                    to_pig_latin_word(&word, &mut output);
                    word.clear();
                }
                output.push(c);
            }
        }
    }
    println!("{}", output);
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
