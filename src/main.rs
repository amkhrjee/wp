use crossterm::style::{Attribute, Color, PrintStyledContent, SetForegroundColor, Stylize};
use crossterm::terminal::{size, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue};
use std::fmt::format;
use std::io::{stdout, BufRead, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    // let args: Vec<_> = env::args().collect();

    // println!("here are the args:");
    // for arg in args {
    //     println!("{arg}");
    // }

    // let test_link = "https://en.wikipedia.org/wiki/Alan_Turing".to_string();
    // let test_link = "https://en.wikipedia.org/wiki/Miss_Meyers".to_string();
    // let test_link = "https://en.wikipedia.org/wiki/Konnagar".to_string();
    let test_link = "https://en.wikipedia.org/wiki/South_Suburban_School_(Main)".to_string();
    // let test_link = "https://en.wikipedia.org/wiki/Luchi".to_string();
    // let test_link = "https://en.wikipedia.org/wiki/Ol%C3%A9".to_string();
    // let test_link = "https://en.wikipedia.org/wiki/Premendra_Mitra".to_string();
    // What I have to do:
    // - parse the name out of it
    // - parse the language out of it (future)

    let path_buf = PathBuf::from(test_link);
    let title = path_buf.file_name().unwrap().to_str().unwrap();

    let url = format!("https://en.wikipedia.org/w/api.php?action=query&format=json&prop=revisions&titles={title}&formatversion=2&rvprop=content&rvslots=*");

    let (title, content) = get_content(url).unwrap();
    let content_len = content.len();
    let content = &content[1..content_len - 1].to_string();
    let title_len = title.len();
    let title = &title[1..title_len - 1];
    parse_content(&title, content);
}

fn get_content(url: String) -> Result<(String, String), reqwest::Error> {
    let res: serde_json::Value = reqwest::blocking::get(url)?.json()?;

    let title = res["query"]["pages"][0]["title"].to_string();

    let content = res["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"].to_string();

    Ok((title, content))
}
#[derive(Debug, PartialEq, Clone, Copy)]
enum FormatType {
    // URL,
    // Code,
    Bold,
    Title,
    Italic,
    PlainWord,
    Space,
    Period,
    Subtitle,
    Subsubtitle,
    ItalicWikiLink,
    WikiLink,
    NewLine,
    // Citation,
    // YearSpan,
    // CodeSnippet,
    // PostNominal,
    BulletBold,
    BulletItalic,
    BulletPlain,
    InlineQuote,
}

struct Token {
    start: usize,
    length: usize,
    format: FormatType,
}

impl Token {
    fn print(&self, source: &[char]) {
        println!(
            "FormatType: {:?} | {}",
            self.format,
            source[self.start..self.start + self.length]
                .iter()
                .collect::<String>()
        )
    }

    fn to_string(&self, source: &[char]) -> String {
        source[self.start..self.start + self.length]
            .iter()
            .collect::<String>()
            .trim()
            .to_string()
    }
}

fn parse_content(title: &str, content: &String) {
    let mut source = Vec::new();

    for character in content.chars() {
        source.push(character);
    }

    let source = &source;

    let mut start;
    let mut current = 0;

    let mut tokens = Vec::new();

    let mut is_bullet = false;

    while current < source.len() {
        match source[current] {
            '{' => {
                // Assuming we can only have three levels of nesting
                current += 2;
                while advance(source, &mut current) != '}' {
                    if source[current] == '{' {
                        current += 1;
                        while advance(source, &mut current) != '}' {
                            if source[current] == '{' {
                                current += 1;
                                while advance(source, &mut current) != '}' {}
                                current += 1;
                            }
                        }
                        current += 1;
                    }
                }
                current += 1;
            }
            '[' => {
                // There are many possibilities here
                // If it has nesting - then we completely ignore this
                // otherwise we check for links [todo]
                current += 2;
                while advance(source, &mut current) != ']' {
                    if source[current] == '[' {
                        while advance(source, &mut current) != ']' {}
                        current += 1;
                    }
                }
                current += 1;
            }
            '<' => while advance(source, &mut current) != '>' {},
            '=' => {
                let mut equals_count = 0;
                while advance(source, &mut current) == '=' {
                    equals_count += 1;
                }
                start = current - 1;
                while advance(source, &mut current) != '=' {}
                match equals_count {
                    2 => {
                        tokens.push(make_token(start, current - start - 1, FormatType::Title));
                        current += 1;
                    }
                    3 => {
                        tokens.push(make_token(start, current - start - 1, FormatType::Subtitle));
                        current += 2;
                    }
                    4 => {
                        tokens.push(make_token(
                            start,
                            current - start - 1,
                            FormatType::Subsubtitle,
                        ));
                        current += 3;
                    }
                    _ => {}
                }
            }
            '\\' => {
                current += 1;
                if source[current] == 'n' {
                    start = current - 1;
                    tokens.push(make_token(start, 2, FormatType::NewLine));
                    current += 1;
                } else if source[current] == '"' {
                    current += 1;
                    start = current;
                    while advance(source, &mut current) != '\\' {}
                    tokens.push(make_token(
                        start,
                        current - start - 1,
                        FormatType::InlineQuote,
                    ));
                    current += 1;
                } else {
                    current += 1
                }
            }
            '*' => {
                is_bullet = true;
                current += 1;
            }
            '\'' => {
                if source[current + 1] == '\'' {
                    let mut apostrophe_count = 0;
                    let mut format = FormatType::Bold;
                    while advance(source, &mut current) == '\'' {
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
                    while advance(source, &mut current) != '\'' {}
                    tokens.push(make_token(start, current - start - 1, format));
                    current += apostrophe_count - 1;
                } else {
                    current += 1;
                }
            }
            ' ' => {
                tokens.push(make_token(current, 1, FormatType::Space));
                current += 1;
            }
            _ => {
                start = current;
                while !matches!(
                    source[current],
                    '<' | '=' | '{' | '[' | '\\' | '*' | ' ' | '\'' | '\0'
                ) {
                    current += 1;
                }
                tokens.push(make_token(start, current - start, FormatType::PlainWord));
                if source[current] == '\0' {
                    break;
                }
            }
        }
    }
    // for token in tokens {
    //     if token.format != FormatType::NewLine
    //         && token.format != FormatType::WikiLink
    //         && token.format != FormatType::Space
    //     {
    //         token.print(source);
    //     }
    // }
    display(&source, title, &tokens);
}

fn advance(source: &[char], current: &mut usize) -> char {
    *current += 1;
    if *current >= source.len() {
        return '\0';
    }
    return source[*current - 1];
}

fn make_token(start: usize, length: usize, format: FormatType) -> Token {
    Token {
        start,
        length,
        format,
    }
}

fn display(source: &[char], title: &str, tokens: &Vec<Token>) {
    let mut stdout = stdout();
    let mut row_number = 2;
    let mut column_number = 0;
    let (width, _height) = size().unwrap();
    execute!(stdout, EnterAlternateScreen).unwrap();

    let mut has_first_word_rendered = false;

    // // Set the title
    queue!(
        stdout,
        SetForegroundColor(Color::DarkCyan),
        cursor::MoveTo((width - title.len() as u16) / 2, 1),
        PrintStyledContent(
            title
                .attribute(Attribute::Bold)
                .attribute(Attribute::Underlined)
        ),
    )
    .unwrap();
    row_number += 2;

    for token in tokens {
        let token_string = token.to_string(source);
        match token.format {
            // FormatType::URL => todo!(),
            // FormatType::Code => todo!(),
            FormatType::Bold => {
                queue!(
                    stdout,
                    cursor::MoveTo(
                        get_column_number(&mut row_number, &mut column_number),
                        row_number
                    ),
                    PrintStyledContent(
                        token_string
                            .clone()
                            .with(Color::White)
                            .attribute(Attribute::Bold)
                    ),
                )
                .unwrap();
                column_number += token_string.len() as u16;
            }
            FormatType::Italic => {
                queue!(
                    stdout,
                    cursor::MoveTo(
                        get_column_number(&mut row_number, &mut column_number),
                        row_number
                    ),
                    PrintStyledContent(
                        token_string
                            .clone()
                            .with(Color::White)
                            .attribute(Attribute::Italic)
                    ),
                )
                .unwrap();
                column_number += token_string.len() as u16;
            }
            FormatType::Title => {
                queue!(
                    stdout,
                    cursor::MoveTo(0, row_number),
                    PrintStyledContent(
                        token
                            .to_string(source)
                            .with(Color::Green)
                            .attribute(Attribute::Bold)
                    ),
                )
                .unwrap();
                // row_number += 1;
                column_number = 0;
            }
            FormatType::PlainWord => {
                if !has_first_word_rendered {
                    has_first_word_rendered = true;
                }
                queue!(
                    stdout,
                    cursor::MoveTo(
                        get_column_number(&mut row_number, &mut column_number),
                        row_number
                    ),
                    PrintStyledContent(token_string.clone().with(Color::White)),
                )
                .unwrap();
                column_number += token_string.len() as u16;
            }
            FormatType::Space => {
                column_number += 1;
            }
            FormatType::Subtitle => {
                queue!(
                    stdout,
                    cursor::MoveTo(0, row_number),
                    PrintStyledContent(token_string.with(Color::Blue)),
                )
                .unwrap();
                row_number += 1;
                column_number = 0;
            }
            FormatType::Subsubtitle => {
                queue!(
                    stdout,
                    cursor::MoveTo(0, row_number),
                    PrintStyledContent(token_string.with(Color::Cyan)),
                )
                .unwrap();
                row_number += 1;
                column_number = 0;
            }
            FormatType::NewLine => {
                if has_first_word_rendered {
                    row_number += 1;
                    column_number = 0;
                }
            }
            FormatType::InlineQuote => {
                let quote_string = format!("\"{token_string}\"");
                queue!(
                    stdout,
                    cursor::MoveTo(
                        get_column_number(&mut row_number, &mut column_number),
                        row_number
                    ),
                    PrintStyledContent(
                        quote_string
                            .clone()
                            .with(Color::White)
                            .attribute(Attribute::Italic)
                    ),
                )
                .unwrap();
                column_number += quote_string.len() as u16;
            }
            // FormatType::WikiLink => todo!(),
            // FormatType::Citation => todo!(),
            // FormatType::YearSpan => todo!(),
            // FormatType::CodeSnippet => todo!(),
            // FormatType::PostNominal => {}
            // FormatType::BlockQuote => todo!(),
            // FormatType::BulletPoint => todo!(),
            // FormatType::ShortDescription => todo!(),
            _ => {}
        }
    }
    stdout.flush().unwrap();
    thread::sleep(Duration::from_secs(60));
    execute!(stdout, LeaveAlternateScreen).unwrap();
}

fn get_column_number(row_number: &mut u16, column_number: &mut u16) -> u16 {
    let (columns, _) = size().unwrap();
    let initial_column_number = *column_number;
    if *column_number >= columns {
        *column_number = 0;
        *row_number += 1;
    }

    initial_column_number % columns
}
