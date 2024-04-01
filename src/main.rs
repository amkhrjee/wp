use crossterm::terminal::{size, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;
// use std::{env, path::PathBuf};
fn main() {
    // let args: Vec<_> = env::args().collect();

    // println!("here are the args:");
    // for arg in args {
    //     println!("{arg}");
    // }

    // let test_link = "https://en.wikipedia.org/wiki/Rust_(programming_language)".to_string();
    // What I have to do:
    // - parse the name out of it
    // - parse the language out of it (future)

    // let path_buf = PathBuf::from(test_link);
    // let title = path_buf.file_name().unwrap().to_str().unwrap();

    // let api_url = format!("https://en.wikipedia.org/w/api.php?action=query&format=json&prop=revisions&titles={title}&formatversion=2&rvprop=content&rvslots=*");

    // These are all I need for now
    // What I have to do:
    // - Make it take up the full screen of the terminal
    // - Make a parser for the content

    let mut stdout = stdout();
    let (width, height) = size().unwrap();
    execute!(stdout, EnterAlternateScreen).unwrap();

    // queue!(stdout, Clear(ClearType::All)).unwrap();
    queue!(stdout, cursor::MoveTo(width / 2, height / 2)).unwrap();
    stdout.write("urmom so fat".as_bytes()).unwrap();
    stdout.flush().unwrap();
    thread::sleep(Duration::from_secs(5));

    execute!(stdout, LeaveAlternateScreen).unwrap();
}

// fn get_content(url: String) -> Result<(String, String), reqwest::Error> {
//     let res: serde_json::Value = reqwest::blocking::get(url)?.json()?;

//     let title = res["query"]["pages"][0]["title"].to_string();

//     let content = res["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"].to_string();

//     Ok((title, content))
// }
