use std::{
    fs::File,
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, BufRead, Write},
    path::Path,
    sync::{Arc, Mutex},
    thread::spawn,
};

use regex::Regex;

use crate::{plaintext_from_link, FormatType, Token};

pub fn advance(text: &Vec<char>, current: &mut usize, iter: &mut i32) -> char {
    let max = 150000;
    if *iter < max {
        *iter += 1;
        *current += 1;
        if *current < text.len() {
            return text[*current - 1];
        }
        return '\0';
    } else {
        panic!("Infinite loop");
        // exit(1);
    }
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
    // The parser is a hot pile of mess and needs to be rewritten asap
    for token in tokens {
        match token.format {
            FormatType::Title
            | FormatType::Bold
            | FormatType::PlainWord
            | FormatType::Subtitle
            | FormatType::Subsubtitle
            | FormatType::WikiLink
            | FormatType::BulletBold
            | FormatType::BulletItalic => plaintext.push_str(&get_text(token)),
            FormatType::Italic | FormatType::InlineQuote => {
                let text_with_artifact = &get_text(token).replace("[[", "");
                let regex_pattern = Regex::new(r"\|.*?\]\]").unwrap();
                let cleaned_text = regex_pattern
                    .replace_all(text_with_artifact, "")
                    .to_string();
                plaintext.push_str(&cleaned_text.replace("]]", ""));
            }
            FormatType::Space => plaintext.push(' '),
            FormatType::NewLine => plaintext.push('\n'),
        }
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
    let file_path;
    if is_bulk {
        file_path = format!("./wp_downloads/{}", hash);
    } else {
        file_path = format!("{}", hash);
    }
    let path = Path::new(&file_path);

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

pub fn download_from_file(link: &str) -> Option<bool> {
    use indicatif::ProgressBar;
    let mut list_of_links = vec![];
    if let Ok(lines) = read_lines(link) {
        for line in lines.flatten() {
            list_of_links.push(line.trim().to_string());
        }
    }
    let mut handles = vec![];
    let total_count = &list_of_links.len();

    let bar = Arc::new(Mutex::new(ProgressBar::new(
        (*total_count).try_into().unwrap(),
    )));

    println!("🔍 Total links found: {}", total_count);
    println!("🗃️ Downloading articles in bulk in wp_downlods...\n");

    for link in list_of_links {
        let bar = Arc::clone(&bar);
        let handle = spawn(move || {
            let (plaintext, url_title) = plaintext_from_link(&link);
            let mut hasher = DefaultHasher::new();
            save_to_disk(&plaintext, &url_title, &mut hasher, true);
            bar.lock().unwrap().inc(1);
        });
        handles.push(handle);
    }
    // Just pure evil
    for handle in handles {
        match handle.join() {
            Ok(_) => (),
            Err(_) => continue,
        }
    }
    bar.lock().unwrap().finish_and_clear();

    println!("\n✅ Download complete.");
    Some(true)
}
