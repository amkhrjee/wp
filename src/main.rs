use clap::Parser;
use std::{
    hash::DefaultHasher,
    path::Path,
    sync::{Arc, Mutex},
    thread::{self},
};
use url::Url;

use core::*;
use utils::*;
mod core;
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

fn main() {
    let args = Args::parse();
    let link = args.link;
    // Check if the link is a file or a url
    if Url::parse(&link).is_ok() {
        let (plaintext, url_title) = plaintext_from_link(&link);
        if args.save {
            let mut hasher = DefaultHasher::new();
            save_to_disk(&plaintext, &url_title, &mut hasher, false);
        } else {
            output_to_stdout(&plaintext);
        }
    } else if Path::new(&link).exists() {
        use indicatif::ProgressBar;
        let mut list_of_links = vec![];
        if let Ok(lines) = read_lines(&link) {
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
        println!("🗃️ Downloading articles in bulk...");
        for link in list_of_links {
            let bar = Arc::clone(&bar);
            let handle = thread::spawn(move || {
                let (plaintext, url_title) = plaintext_from_link(&link);
                let mut hasher = DefaultHasher::new();
                save_to_disk(&plaintext, &url_title, &mut hasher, true);
                bar.lock().unwrap().inc(1);
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }

        bar.lock().unwrap().finish_and_clear();

        println!("\n✅ Download complete.");
    } else {
        println!("\x1b[31m⚠️ Link provided should be either a URL or a valid file path.\x1b[0m")
    }
}
