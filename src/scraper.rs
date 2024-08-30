use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use clap::Parser;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use url::Url;
use zip::write::SimpleFileOptions;

use crate::download_from_file;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, value_parser = ["as", "hi", "bn", "bh", "ne", "or", "te", "gu", "kn", "mr", "pi", "sa", "ta"])]
    lang: String,

    #[clap(long)]
    save: bool,
}

fn get_links(parsed_html: &Html, main_url: &str, batch_count: usize, lang: &str) -> usize {
    println!("⚡ Starting batch {}", batch_count);

    let ul_selector = Selector::parse("ul.mw-allpages-chunk").unwrap();
    let li_selector = Selector::parse("li").unwrap();
    let a_selector = Selector::parse("a[href]").unwrap();

    let links: Vec<String> = parsed_html
        .select(&ul_selector)
        .flat_map(|ul| ul.select(&li_selector))
        .flat_map(|li| li.select(&a_selector))
        .filter_map(|a| a.value().attr("href"))
        .map(|link| format!("https://{}{}", main_url, link))
        .collect();

    let batch_size = links.len();
    println!("⚡ Accumulated {} links", batch_size);

    let file_name = format!("{}_{}.links", lang, batch_count);
    let file = File::create(&file_name).unwrap();
    let mut writer = BufWriter::new(file);

    for link in &links[..links.len().saturating_sub(1)] {
        writeln!(writer, "{}", link).unwrap();
    }

    if let Some(last) = links.last() {
        write!(writer, "{}", last).unwrap();
    }

    batch_size
}

pub fn bulk_download_or_save_links(
    lang: &str,
    is_links_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_url = match lang {
        "as" => "https://as.wikipedia.org/wiki/%E0%A6%AC%E0%A6%BF%E0%A6%B6%E0%A7%87%E0%A6%B7:%E0%A6%B8%E0%A6%95%E0%A6%B2%E0%A7%8B%E0%A6%AC%E0%A7%8B%E0%A7%B0_%E0%A6%AA%E0%A7%83%E0%A6%B7%E0%A7%8D%E0%A6%A0%E0%A6%BE/%E0%A6%85",
        _ => return Err("Unsupported language".into()),
    };

    let client = Client::new();
    let mut links_count = 0;
    let mut batch_count = 0;

    let url = Url::parse(start_url)?;
    let main_url = url.host_str().ok_or("Invalid URL")?;

    println!("💭 Links will be saved to your current directory as zip.");
    println!("⚡ Scraping links...");

    let mut next_batch_link = start_url.to_string();

    loop {
        let response = client.get(&next_batch_link).send()?;
        let html = response.text()?;
        let parsed_html = Html::parse_document(&html);

        batch_count += 1;
        links_count += get_links(&parsed_html, main_url, batch_count, lang);

        let nav_selector = Selector::parse("div.mw-allpages-nav").unwrap();
        let a_selector = Selector::parse("a[href]").unwrap();

        next_batch_link = parsed_html
            .select(&nav_selector)
            .nth(1)
            .and_then(|div| div.select(&a_selector).nth(1))
            .and_then(|a| a.value().attr("href"))
            .map(|link| format!("https://{}{}", main_url, link))
            .ok_or("No next batch link found")?;

        if next_batch_link.is_empty() {
            break;
        }
    }

    println!("✅ All links saved.");

    if is_links_only {
        println!("🗃️ Zipping up all the links...");
        let zip_file = File::create(format!("{}.zip", lang))?;
        let mut zip = zip::ZipWriter::new(zip_file);

        for entry in std::fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("links") {
                let file_name = &path.file_name().unwrap().to_str().unwrap();
                zip.start_file(
                    file_name,
                    SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Deflated),
                )?;
                let contents = std::fs::read(&path)?;
                zip.write_all(&contents)?;
                std::fs::remove_file(path)?;
            }
        }

        zip.finish()?;
    } else {
        // Download straight from the links!
        println!("⚡ Proceeding with the downloads...");
        let dir_path = Path::new(".");
        let files: Vec<_> = fs::read_dir(dir_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file()
                    && entry.path().extension().and_then(|s| s.to_str()) == Some("links")
            })
            .collect();
        for each_file in files {
            let file_path = each_file.path();
            download_from_file(file_path.to_str().unwrap())
        }
    }

    println!("📊 Total batches done: {}", batch_count);
    println!("🔗 Total links saved: {}", links_count);

    Ok(())
}
