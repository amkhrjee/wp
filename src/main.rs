use clap::{builder::PossibleValue, Parser};
use std::{
    fs::File,
    io::Write,
    os::linux::raw::stat,
    path::{Path, PathBuf},
};
use url::Url;

use utils::*;
mod utils;

/// Simple program to greet a person
#[derive(Parser)]
#[command(about = "Wikipedia on your terminal.")]
#[command(version, long_about = None)]
struct Args {
    #[arg(short, long, help = "link to the wikipedia article")]
    link: String,

    #[arg(short, long, help = "save to disk", action)]
    save: bool,
}

#[derive(Debug)]
enum FormatType {
    Bold,
    Title,
    Italic,
    PlainWord,
    Space,
    Subtitle,
    Subsubtitle,
    WikiLink,
    NewLine,
    BulletBold,
    BulletItalic,
    InlineQuote,
}

struct Token {
    start: usize,
    length: usize,
    format: FormatType,
}

impl Token {
    fn print(&self, source: &Vec<char>) {
        println!(
            "{:?} => {}",
            self.format,
            source[self.start..self.start + self.length]
                .iter()
                .collect::<String>()
        )
    }
}

fn main() {
    let args = Args::parse();
    let link = args.link;
    let path_buf = PathBuf::from(&link);
    // well, if we can't get the name, just panic and quit!
    let url_title = path_buf
        .file_name()
        .unwrap()
        .to_str()
        .unwrap_or_else(|| panic!("Error: could not get name of article."));

    let url = Url::parse(&link).unwrap();
    let wikipedia_url = url.host_str().unwrap();

    let raw_text = get_article(format!("https://{wikipedia_url}/w/api.php?action=query&format=json&prop=revisions&titles={url_title}&formatversion=2&rvprop=content&rvslots=*")).unwrap();
    // Trimming the fat
    let slice_index = raw_text.find("\"").unwrap();
    let mut characters: Vec<char> = raw_text[slice_index + 1..].chars().collect();
    characters.pop();
    characters.push('\0');
    let tokens = parse_text(&characters).unwrap();
    let plaintext = generate_plaintext(&tokens, &characters);
    if args.save {
        save_to_disk(&plaintext, &(url_title.to_string() + ".txt"));
    } else {
        output_to_stdout(&plaintext);
    }
}

fn get_article(url: String) -> Result<(String), String> {
    let response: serde_json::Value = reqwest::blocking::get(url)
        .map_err(|err| format!("Error: Could not fetch article due to {}", err))?
        .json()
        .map_err(|err| format!("Error: JSON conversion failed due to {}", err))?;
    Ok(
        // response["query"]["pages"][0]["title"].to_string(),
        response["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"].to_string(),
    )
}

// little utils : move to a separate file

fn parse_text(characters: &Vec<char>) -> Result<Vec<Token>, String> {
    let mut start: usize;
    let mut current = 0;
    let mut tokens: Vec<Token> = Vec::new();
    let mut is_bullet = false;

    // Removing the Infobox stuff (let's call it "Prelude")
    // Assumption: the first word is always bold
    while current < characters.len() {
        match characters[current] {
            '{' => {
                // Assuming we can only have three levels of nesting
                // This is some convoluted shit thanks to wikipedia's format:(
                current += 2;
                while advance(characters, &mut current) != '}' {
                    if characters[current] == '{' {
                        current += 1;
                        while advance(characters, &mut current) != '}' {
                            if characters[current] == '{' {
                                current += 1;
                                while advance(characters, &mut current) != '}' {}
                                current += 1;
                            }
                        }
                        current += 1;
                    }
                }
                current += 1;
            }
            '\'' => {
                if peek_ahead(&characters, current) == '\'' {
                    let mut apostrophe_count = 0;
                    let mut format = FormatType::Bold;
                    while advance(&characters, &mut current) == '\'' {
                        apostrophe_count += 1;
                    }
                    if apostrophe_count == 2 {
                        if is_bullet {
                            format = FormatType::BulletItalic;
                            is_bullet = false;
                        } else {
                            format = FormatType::Italic;
                        }
                    } else if apostrophe_count == 3 {
                        if is_bullet {
                            format = FormatType::BulletBold;
                            is_bullet = false;
                        }
                    }
                    start = current - 1;

                    while advance(&characters, &mut current) != '\'' {}
                    add_token(&mut tokens, start, current, format);
                    current += apostrophe_count - 1;
                } else {
                    current += 1;
                }
            }
            '[' => {
                // There are many possibilities here
                // If it has nesting - then we completely ignore this
                // otherwise we check for links [todo]
                let mut has_nesting = false;
                let mut has_pipe = false;
                current += 2;
                start = current;
                while advance(characters, &mut current) != ']' {
                    if characters[current] == '[' {
                        has_nesting = true;
                        while advance(characters, &mut current) != ']' {}
                        current += 1;
                    } else if characters[current] == '|' {
                        has_pipe = true;
                    }
                }

                if !has_nesting && !has_pipe {
                    add_token(&mut tokens, start, current, FormatType::WikiLink);
                }
                // TODO: handle the pipe case

                current += 1;
            }
            ' ' => {
                add_space(&mut tokens, current);
                current += 1;
            }
            '<' => while advance(characters, &mut current) != '>' {},
            '=' => {
                let mut equals_count = 0;
                while advance(characters, &mut current) == '=' {
                    equals_count += 1;
                }
                start = current - 1;
                while advance(characters, &mut current) != '=' {}
                match equals_count {
                    2 => {
                        add_token(&mut tokens, start, current, FormatType::Title);
                        current += 1;
                    }
                    3 => {
                        add_token(&mut tokens, start, current, FormatType::Subtitle);
                        current += 2;
                    }
                    4 => {
                        add_token(&mut tokens, start, current, FormatType::Subsubtitle);
                        current += 3;
                    }
                    _ => {}
                }
            }
            '\\' => {
                current += 1;
                if characters[current] == 'n' {
                    add_new_line(&mut tokens, current);
                } else if characters[current] == '"' {
                    current += 1;
                    start = current;
                    while advance(characters, &mut current) != '\\' {}
                    add_token(&mut tokens, start, current, FormatType::InlineQuote);
                }
                current += 1;
            }
            '*' => {
                is_bullet = true;
                current += 1;
            }
            _ => {
                if current >= characters.len() {
                    break;
                }
                start = current;
                while !matches!(
                    characters[current],
                    '<' | '=' | '{' | '[' | '\\' | '*' | ' ' | '\'' | '\0'
                ) {
                    current += 1;
                }
                tokens.push(Token {
                    start,
                    length: current - start,
                    format: FormatType::PlainWord,
                });
                if characters[current] == '\0' {
                    break;
                }
            }
        }
    }

    Ok(tokens)
}

fn generate_plaintext(tokens: &Vec<Token>, characters: &Vec<char>) -> String {
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

fn output_to_stdout(plaintext_string: &str) {
    println!("{}", plaintext_string);
}

fn save_to_disk(plaintext_string: &str, article_title: &str) {
    let path = Path::new(article_title);

    let mut file = match File::create(&path) {
        Err(why) => panic!("Error: Couldn't create {}: {}", path.display(), why),
        Ok(file) => file,
    };

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(plaintext_string.as_bytes()) {
        Err(why) => panic!("Error: Couldn't write to {}: {}", path.display(), why),
        Ok(_) => println!("Saved to {}", path.display()),
    }
}
