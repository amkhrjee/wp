use std::{env, path::PathBuf};
fn main() -> Result<(), reqwest::Error> {
    let args: Vec<_> = env::args().collect();

    println!("here are the args:");
    for arg in args {
        println!("{arg}");
    }

    let test_link = "https://en.wikipedia.org/wiki/Rust_(programming_language)".to_string();
    // What I have to do:
    // - parse the name out of it
    // - parse the language out of it (future)

    let path_buf = PathBuf::from(test_link);
    let title = path_buf.file_name().unwrap().to_str().unwrap();

    let api_url = format!("https://en.wikipedia.org/w/api.php?action=query&format=json&prop=revisions&titles={title}&formatversion=2&rvprop=content&rvslots=*");

    let res: serde_json::Value = reqwest::blocking::get(api_url)?.json()?;

    // These are all I need for now
    let title = res["query"]["pages"][0]["title"].to_string();

    let content = res["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"].to_string();

    // What I have to do:
    // - Make it take up the full screen of the terminal
    // - Make a parser for the content
    println!(
        "{}",
        res["query"]["pages"][0]["revisions"][0]["slots"]["main"]["content"]
    );

    Ok(())
}
