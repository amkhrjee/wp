use std::{
    fs::File,
    hash::{Hash, Hasher},
    io::{self, BufRead, Write},
    path::Path,
};

use crate::{FormatType, Token};

pub fn advance(text: &Vec<char>, current: &mut usize) -> char {
    *current += 1;
    if *current < text.len() {
        return text[*current - 1];
    }
    return '\0';
}

pub fn add_token(tokens: &mut Vec<Token>, start: usize, current: usize, format: FormatType) {
    tokens.push(Token {
        start,
        length: current - start - 1,
        format,
    });
}

pub fn add_new_line(tokens: &mut Vec<Token>, current: usize) {
    tokens.push(Token {
        start: current - 1,
        length: 2,
        format: FormatType::NewLine,
    })
}

pub fn peek_ahead(text: &Vec<char>, current: usize) -> char {
    if current + 1 < text.len() {
        return text[current + 1];
    }
    return '\0';
}

pub fn add_space(tokens: &mut Vec<Token>, current: usize) {
    tokens.push(Token {
        start: current,
        length: 1,
        format: FormatType::Space,
    })
}

pub fn generate_plaintext(tokens: &Vec<Token>, characters: &Vec<char>) -> String {
    let mut plaintext = String::new();
    let get_text = |token: &Token| {
        characters[token.start..token.start + token.length]
            .iter()
            .collect::<String>()
    };
    for token in tokens {
        match token.format {
            FormatType::Title
            | FormatType::Bold
            | FormatType::PlainWord
            | FormatType::Subtitle
            | FormatType::Subsubtitle
            | FormatType::WikiLink
            | FormatType::BulletBold
            | FormatType::BulletItalic
            | FormatType::InlineQuote => plaintext.push_str(&get_text(token)),

            FormatType::Italic => {
                let raw_token_text = &get_text(token);
                if raw_token_text.find("[[").is_some() && raw_token_text.find("]]").is_some() {
                    match raw_token_text.find("|") {
                        Some(index) => plaintext.push_str(&raw_token_text[2..index]),
                        None => plaintext.push_str(&raw_token_text[2..token.length - 2]),
                    }
                } else {
                    plaintext.push_str(&raw_token_text)
                }
            }
            FormatType::Space => plaintext.push(' '),
            FormatType::NewLine => plaintext.push('\n'),
        }
        // println!("{:?}: {}", token.format, &get_text(token));
    }
    plaintext.trim().to_string()
}

pub fn output_to_stdout(plaintext_string: &str) {
    println!("{}", plaintext_string);
}

pub fn save_to_disk(
    plaintext_string: &str,
    article_title: &str,
    hasher: &mut impl Hasher,
    is_bulk: bool,
) {
    article_title.hash(hasher);
    let hash = hasher.finish();
    let hash = format!("{:x}.txt", hash);
    let path = Path::new(&hash);

    let mut file = match File::create(&path) {
        Err(why) => panic!("Error: Couldn't create {}: {}", path.display(), why),
        Ok(file) => file,
    };

    match file.write_all(plaintext_string.as_bytes()) {
        Err(why) => panic!("Error: Couldn't write to {}: {}", path.display(), why),
        Ok(_) => {
            if !is_bulk {
                println!("\x1B[32mSaved to {}\x1B[0m", path.display())
            }
        }
    }
}

// Stolen straight from Rust by Examples :P
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
