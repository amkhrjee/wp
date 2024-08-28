use std::{fs::File, io::Write, path::Path};

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
            | FormatType::Italic
            | FormatType::PlainWord
            | FormatType::Subtitle
            | FormatType::Subsubtitle
            | FormatType::WikiLink
            | FormatType::BulletBold
            | FormatType::BulletItalic
            | FormatType::InlineQuote => plaintext.push_str(&get_text(token)),

            FormatType::Space => plaintext.push(' '),
            FormatType::NewLine => plaintext.push('\n'),
        }
    }
    plaintext.trim().to_string()
}

pub fn output_to_stdout(plaintext_string: &str) {
    println!("{}", plaintext_string);
}

pub fn save_to_disk(plaintext_string: &str, article_title: &str) {
    let path = Path::new(article_title);

    let mut file = match File::create(&path) {
        Err(why) => panic!("Error: Couldn't create {}: {}", path.display(), why),
        Ok(file) => file,
    };

    match file.write_all(plaintext_string.as_bytes()) {
        Err(why) => panic!("Error: Couldn't write to {}: {}", path.display(), why),
        Ok(_) => println!("Saved to {}", path.display()),
    }
}
