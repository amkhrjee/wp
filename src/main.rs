use crossterm::style::{Attribute, Color, PrintStyledContent, SetForegroundColor, Stylize};
use crossterm::terminal::{size, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue};
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

fn main() {
    // let args: Vec<_> = env::args().collect();

    // println!("here are the args:");
    // for arg in args {
    //     println!("{arg}");
    // }

    let test_link = "https://en.wikipedia.org/wiki/Hurricane_Irene_(2005)".to_string();
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
#[derive(Debug, PartialEq)]
enum FormatType {
    // URL,
    // Code,
    Bold,  //done
    Title, //done
    PlainSentence,
    Subtitle,    //done
    Subsubtitle, //done
    // WikiLink,
    // Citation,
    // YearSpan,
    // CodeSnippet,
    // PostNominal,
    // BlockQuote,
    InlineQuote,
    BulletPoint, //done
}

struct Token {
    start: usize,
    length: usize,
    format_type: FormatType,
}

impl Token {
    fn print(&self, source: &[char]) {
        println!(
            "FormatType: {:?} | {}",
            self.format_type,
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

    source.push('\0');

    let mut start = 0;
    let mut current = 0;
    let mut tokens = Vec::new();

    while current < source.len() {
        if matches_pattern(&source, &"==".to_string(), &mut current) {
            if source[current] == '=' {
                if source[current + 1] == '=' {
                    current += 2;
                    start = current;
                    while source[current] != '=' {
                        advance(&mut current);
                    }
                    tokens.push(Token {
                        start,
                        length: current - start,
                        format_type: FormatType::Subsubtitle,
                    });
                    current += 4;
                } else {
                    advance(&mut current);
                    start = current;
                    while source[current] != '=' {
                        advance(&mut current);
                    }
                    tokens.push(Token {
                        start,
                        length: current - start,
                        format_type: FormatType::Subtitle,
                    });
                    current += 4;
                }
            } else {
                start = current;
                while source[current] != '=' {
                    advance(&mut current);
                }
                tokens.push(Token {
                    start,
                    length: current - start,
                    format_type: FormatType::Title,
                });
                current += 2;
            }
        } else if matches_pattern(&source, &"'''".to_string(), &mut current) {
            start = current;
            while source[current] != '\'' {
                advance(&mut current);
            }
            tokens.push(Token {
                start,
                length: current - start,
                format_type: FormatType::Bold,
            });
            current += 3;
        } else if matches_pattern(&source, &"<ref ".to_string(), &mut current) {
            // println!("Caught a reference!!!!");
            // println!("Current Char: {}", source[current]);
            while !matches!(source[current], '<' | '/') {
                advance(&mut current);
            }
            if source[current] == '/' {
                current += 2;
            } else {
                current += 6;
            }
        } else if matches_pattern(&source, &"{{".to_string(), &mut current) {
            while source[current] != '}' {
                advance(&mut current);
                if matches!(source[current], '{') {
                    while source[current] != '}' {
                        advance(&mut current);
                    }
                    current += 2;
                }
            }
            current += 2;
        } else if matches_pattern(&source, &"[[".to_string(), &mut current) {
            while source[current] != ']' {
                advance(&mut current);
            }
            current += 2;
        } else if matches!(source[current], '*') {
            advance(&mut current);
            start = current;
            while !matches!(source[current], '\\') {
                advance(&mut current);
            }
            tokens.push(Token {
                start,
                length: current - start,
                format_type: FormatType::BulletPoint,
            });
            current -= 1;
        } else if matches!(source[current], '\\') {
            advance(&mut current);
            if matches!(source[current], 'n') {
                advance(&mut current);
                if source[current] == '\\' {
                    current += 2;
                }
            } else if matches!(source[current], '"') {
                start = current;
                while !matches!(source[current], '\\') {
                    advance(&mut current);
                }
                tokens.push(Token {
                    start,
                    length: current - start,
                    format_type: FormatType::InlineQuote,
                });
                current += 2;
            }
        } else if matches!(source[current], ' ') {
            advance(&mut current);
        } else {
            start = current;
            while !matches!(source[current], '.' | '\0' | '<' | ',') {
                if !matches!(source[current], '.' | '<' | ',') {
                    advance(&mut current);
                }
            }
            tokens.push(Token {
                start,
                length: current - start,
                format_type: FormatType::PlainSentence,
            });

            advance(&mut current);
        }
    }

    for token in &tokens {
        // if token.format_type == FormatType::Title {
        token.print(&source);
        // }
    }
    // display(&source, title, &tokens);
}

fn matches_pattern(source: &[char], pattern: &String, current: &mut usize) -> bool {
    let pattern_len = pattern.len();
    let end_index = *current + pattern_len - 1;
    if end_index >= source.len() {
        return false;
    }
    let source_string = source[*current..end_index + 1].iter().collect::<String>();
    if source_string == *pattern {
        *current = end_index + 1;
        return true;
    }
    return false;
}

fn advance(current: &mut usize) {
    *current += 1;
}

fn display(source: &[char], title: &str, tokens: &Vec<Token>) {
    let mut stdout = stdout();
    let mut row = 2;
    let (width, _height) = size().unwrap();
    execute!(stdout, EnterAlternateScreen).unwrap();

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

    for token in tokens {
        match token.format_type {
            // FormatType::URL => todo!(),
            // FormatType::Code => todo!(),
            // FormatType::Bold => todo!(),
            FormatType::Title => {
                queue!(
                    stdout,
                    SetForegroundColor(Color::Blue),
                    cursor::MoveTo(1, row)
                )
                .unwrap();
                stdout.write(token.to_string(source).as_bytes()).unwrap();
                row += 1;
            } // FormatType::PlainSentence => todo!(),
            // FormatType::Subtitle => todo!(),
            // FormatType::Subsubtitle => todo!(),
            // FormatType::WikiLink => todo!(),
            // FormatType::Citation => todo!(),
            // FormatType::YearSpan => todo!(),
            // FormatType::CodeSnippet => todo!(),
            // FormatType::PostNominal => todo!(),
            // FormatType::BlockQuote => todo!(),
            // FormatType::BulletPoint => todo!(),
            // FormatType::ShortDescription => todo!(),
            _ => {}
        }
    }
    stdout.flush().unwrap();
    thread::sleep(Duration::from_secs(5));
    execute!(stdout, LeaveAlternateScreen).unwrap();
}
