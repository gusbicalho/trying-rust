pub const NAME: &str = "Pig Latin";

// Convert strings to pig latin.
// The first consonant of each word is moved to the end of the word and
// “ay” is added, so “first” becomes “irst-fay.”
// Words that start with a vowel have “hay” added to the end instead (“apple”
// becomes “apple-hay”). Keep in mind the details about UTF-8 encoding!

use std::io::{self, Write};

pub fn run() {
    println!("Input a message:");
    let input = util::with_write_buffer_(|buf| {
        io::stdin().read_line(buf).expect("Stdin failure");
    });
    util::transform_words(to_pig_latin_word, &input, &mut io::stdout().lock())
        .expect("Stdout failure");
}

fn to_pig_latin_word<Out: Write>(word: &str, output: &mut Out) -> io::Result<()> {
    fn is_vowel(c: char) -> bool {
        "AEIOUaeiou".contains(c)
    }
    let mut chars = word.chars();
    if let Some(first_char) = chars.next() {
        if is_vowel(first_char) {
            output.write_fmt(format_args!("{}{}-hay", first_char, chars.as_str()))?;
        } else {
            output.write_fmt(format_args!("{}-{}ay", chars.as_str(), first_char))?;
        }
    }
    Result::Ok(())
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

    use std::io::{self, Write};

    pub fn transform_words<Out: Write>(
        transform: fn(&str, &mut Out) -> io::Result<()>,
        input: &str,
        output: &mut Out,
    ) -> io::Result<()> {
        let mut word_start = None;
        for (c_pos, c) in input.char_indices() {
            if c.is_alphabetic() {
                if word_start.is_none() {
                    word_start = Some(c_pos);
                }
            } else {
                if let Some(word_start_pos) = word_start {
                    transform(&input[word_start_pos..c_pos], output)?;
                    word_start = None;
                }
                output.write_fmt(format_args!("{}", c))?;
            }
        }
        Result::Ok(())
    }
}
