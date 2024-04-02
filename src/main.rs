use crossterm::cursor::SetCursorStyle;
use crossterm::style::{Attribute, Color, PrintStyledContent, SetForegroundColor, Stylize};
use crossterm::terminal::{size, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue};
use std::env::current_dir;
use std::io::{stdout, Write};
use std::time::Duration;
use std::{env, path::PathBuf};
use std::{thread, vec};

fn main() {
    // let args: Vec<_> = env::args().collect();

    // println!("here are the args:");
    // for arg in args {
    //     println!("{arg}");
    // }

    let test_link = "https://en.wikipedia.org/wiki/Konnagar".to_string();
    // What I have to do:
    // - parse the name out of it
    // - parse the language out of it (future)

    let path_buf = PathBuf::from(test_link);
    let title = path_buf.file_name().unwrap().to_str().unwrap();

    let url = format!("https://en.wikipedia.org/w/api.php?action=query&format=json&prop=revisions&titles={title}&formatversion=2&rvprop=content&rvslots=*");

    // These are all I need for now
    // What I have to do:
    // - Make it take up the full screen of the terminal
    // - Make a parser for the content

    let (title, content) = get_content(url).unwrap();
    let content_len = content.len();
    let content = &content[1..content_len - 1].to_string();
    parse_content(content);
    // let title_len = title.len();
    // let title = &title[1..title_len - 1];

    // let mut stdout = stdout();
    // let (width, _height) = size().unwrap();
    // execute!(stdout, EnterAlternateScreen).unwrap();

    // // // Set the title
    // queue!(
    //     stdout,
    //     SetForegroundColor(Color::DarkCyan),
    //     cursor::MoveTo((width - title.len() as u16) / 2, 1),
    //     PrintStyledContent(
    //         title
    //             .attribute(Attribute::Bold)
    //             .attribute(Attribute::Underlined)
    //     ),
    // )
    // .unwrap();
    // // stdout.write(title.as_bytes()).unwrap();
    // stdout.flush().unwrap();
    // thread::sleep(Duration::from_secs(5));

    // execute!(stdout, LeaveAlternateScreen).unwrap();
}

fn get_content(url: String) -> Result<(String, String), reqwest::Error> {
    let res: serde_json::Value = reqwest::blocking::get(url)?.json()?;

    let title = res["query"]["pages"][0]["title"].to_string();

    let content = res["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"].to_string();

    Ok((title, content))
}
#[derive(Debug)]
enum FormatType {
    URL,
    Code,
    Bold,  //done
    Title, //done
    PlainSentence,
    Subtitle, //done
    Subsubtitle,
    WikiLink, //done
    Citation, //done
    YearSpan,
    CodeSnippet,
    PostNominal,
    BlockQuote,
    BulletPoint,
    ShortDescription, //done
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
}

fn parse_content(content: &String) {
    let mut source = Vec::new();

    for character in content.chars() {
        source.push(character);
    }

    source.push('\0');

    println!("Last Character: {}", source[source.len() - 1]);

    let mut start = 20;
    let mut current = 0;
    let mut tokens = Vec::new();

    while !is_at_end(&source, current) {
        if source[current] != '}' {
            current += 1;
            continue;
        }
        break;
    }

    tokens.push(Token {
        start: 20,
        length: current - start,
        format_type: FormatType::ShortDescription,
    });

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
                current += 3;
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
        } else if matches_pattern(&source, &"<ref".to_string(), &mut current) {
            while matches_pattern(&source, &"/ref>".to_string(), &mut current) {
                advance(&mut current);
            }
            advance(&mut current);
        } else if matches_pattern(&source, &"{{".to_string(), &mut current) {
            while source[current] != '}' {
                advance(&mut current);
            }
            current += 2;
        } else if matches_pattern(&source, &"[[".to_string(), &mut current) {
            while source[current] != ']' {
                advance(&mut current);
            }
            current += 2;
        } else if matches!(source[current], '*') {
            // println!("Current character: {}", source[current]);
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
        } else if matches_pattern(&source, &"\\n".to_string(), &mut current) {
            if source[current] == '\\' {
                current += 2;
            }
        } else {
            advance(&mut current);
        }
    }

    for token in tokens {
        token.print(&source);
    }
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

fn is_at_end(source: &[char], current: usize) -> bool {
    source.len() == current
}

fn advance(current: &mut usize) {
    *current += 1;
}
