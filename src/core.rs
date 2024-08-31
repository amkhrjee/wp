use std::path::PathBuf;

use regex::Regex;
use url::Url;

use crate::{add_new_line, add_space, add_token, advance, generate_plaintext, peek_ahead};

#[derive(Debug)]
pub enum FormatType {
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

pub struct Token {
    pub start: usize,
    pub length: usize,
    pub format: FormatType,
}

fn remove_nested_braces(input: &str) -> String {
    let mut result = String::new();
    let mut stack = 0;

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            stack += 1;
        } else if c == '}' && chars.peek() == Some(&'}') {
            chars.next();
            if stack > 0 {
                stack -= 1;
            }
        } else if stack == 0 {
            result.push(c);
        }
    }

    result
}

pub fn plaintext_from_link(link: &str) -> (String, String) {
    let path_buf = PathBuf::from(link);
    // well, if we can't get the name, just panic and quit!
    let url_title = path_buf
        .file_name()
        .expect("Could not get parse name of the article.")
        .to_str()
        .expect("Article name is not valid UTF-8");

    let url = Url::parse(link).expect("Invalid URL.");
    let wikipedia_url = url.host_str().expect("Could not get the domain.");

    let raw_text = get_article(format!("https://{wikipedia_url}/w/api.php?action=query&format=json&prop=revisions&titles={url_title}&formatversion=2&rvprop=content&rvslots=*")).unwrap();

    // Trimming out the references
    let re = Regex::new(r"<ref>.*?</ref>").unwrap();
    let mut raw_text = re.replace_all(&raw_text, "").to_string();

    if wikipedia_url.contains("en.wiki") {
        // Trimming out reference for now
        raw_text = regex::Regex::new(r"== References ==.*")
            .unwrap()
            .replace(&raw_text, "")
            .to_string();
    }

    // Trimming out the squigglies
    let raw_text = remove_nested_braces(&raw_text);
    // Trimming out infobox
    let mut characters: Vec<char> = raw_text.find("\"").map_or_else(
        || raw_text.chars().collect(),
        |index| raw_text[index + 1..].chars().collect(),
    );

    characters.pop();
    characters.push('\0');
    let tokens = parse_text(&characters).expect("Failed to parse text.");
    let plaintext = generate_plaintext(&tokens, &characters);
    (plaintext, url_title.to_string())
}

fn parse_text(characters: &Vec<char>) -> Option<Vec<Token>> {
    let mut start: usize;
    let mut current = 0;
    let mut tokens: Vec<Token> = Vec::new();
    let mut is_bullet = false;
    // Very nutty, bad will have to do for now
    const MAX_ITERATINS: i32 = 150000;
    let mut iter_count = 0;

    while current < characters.len() && iter_count < MAX_ITERATINS {
        iter_count += 1;
        match characters[current] {
            '{' => {
                // Assuming we can only have three levels of nesting
                // This is some convoluted shit thanks to wikipedia's format:(
                current += 2;
                while advance(characters, &mut current, &mut iter_count) != '}' {
                    if characters[current] == '{' {
                        current += 1;
                        while advance(characters, &mut current, &mut iter_count) != '}' {
                            if characters[current] == '{' {
                                current += 1;
                                while advance(characters, &mut current, &mut iter_count) != '}' {}
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
                    while advance(&characters, &mut current, &mut iter_count) == '\'' {
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

                    while advance(&characters, &mut current, &mut iter_count) != '\'' {}
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
                while advance(characters, &mut current, &mut iter_count) != ']' {
                    if characters[current] == '[' {
                        has_nesting = true;
                        while advance(characters, &mut current, &mut iter_count) != ']' {}
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
            '<' => while advance(characters, &mut current, &mut iter_count) != '>' {},
            '=' => {
                let mut equals_count = 0;
                while advance(characters, &mut current, &mut iter_count) == '=' {
                    equals_count += 1;
                }
                start = current - 1;
                while advance(characters, &mut current, &mut iter_count) != '=' {}
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
                    while advance(characters, &mut current, &mut iter_count) != '\\' {}
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

    Some(tokens)
}

fn get_article(url: String) -> Result<String, String> {
    let response: serde_json::Value = reqwest::blocking::get(url)
        .map_err(|err| format!("Error: Could not fetch article due to {}", err))?
        .json()
        .map_err(|err| format!("Error: JSON conversion failed due to {}", err))?;
    Ok(
        // response["query"]["pages"][0]["title"].to_string(),
        response["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"].to_string(),
    )
}
