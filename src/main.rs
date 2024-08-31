use clap::Parser;
use scraper::bulk_download_or_save_links;
use std::{hash::DefaultHasher, path::Path};
use url::Url;

use core::*;
use utils::*;
mod core;
mod scraper;
mod utils;

#[derive(Parser)]
#[command(
    about = "Wikipedia on your terminal. Made by Aniruddha <amkhrjee@gmail.com>. Licensed under GPLv3."
)]
#[command(version, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        help = "Link to the wikipedia article or Path to file with links"
    )]
    link: Option<String>,
    #[arg(short, long, help = "Save articles to disk", action)]
    save: bool,

    #[arg(long, value_parser = ["as", "hi", "bn", "bh", "ne", "or", "te", "gu", "kn", "mr", "pi", "sa", "ta"], help="Choose Wikipedia labguage edition for bulk download")]
    lang: Option<String>,

    #[arg(long, help = "Only save the aggregated links to articles.")]
    links_only: bool,
}

fn main() {
    let args = Args::parse();
    if args.lang.is_some() {
        bulk_download_or_save_links(&args.lang.unwrap(), args.links_only)
            .expect("Failed to download articles.");
    } else if args.link.is_some() {
        let link = args
            .link
            .expect("Must provide link if not bulk downloading.");
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
            download_from_file(&link);
        } else {
            println!("\x1b[31m⚠️ Link provided should be either a URL or a valid file path.\x1b[0m")
        }
    } else {
        println!("\x1b[31m⚠️ Invalid arguments. Type wp --help to see all set of options.\x1b[0m")
    }
}
