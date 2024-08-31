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
    let li_selector = Selector::parse("li:not([class])").unwrap();
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
        "bn" => "https://bn.wikipedia.org/wiki/%E0%A6%AC%E0%A6%BF%E0%A6%B6%E0%A7%87%E0%A6%B7:%E0%A6%B8%E0%A6%AC_%E0%A6%AA%E0%A6%BE%E0%A6%A4%E0%A6%BE/%E0%A6%85",
        "hi" => "https://hi.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:%E0%A4%B8%E0%A4%AD%E0%A5%80_%E0%A4%AA%E0%A5%83%E0%A4%B7%E0%A5%8D%E0%A4%A0/%E0%A4%85",
        "bh" => "https://bh.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:%E0%A4%B8%E0%A4%BE%E0%A4%B0%E0%A4%BE_%E0%A4%AA%E0%A4%A8%E0%A5%8D%E0%A4%A8%E0%A4%BE?from=%E0%A4%85&to=&namespace=0",
        "ne" => "https://ne.wikipedia.org/w/index.php?title=%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:AllPages&from=%E0%A4%85%E0%A4%A4%E0%A4%BF%E0%A4%AF%E0%A4%A5%E0%A4%BE%E0%A4%B0%E0%A5%8D%E0%A4%A5%E0%A4%B5%E0%A4%BE%E0%A4%A6",
        "or" => "https://or.wikipedia.org/wiki/%E0%AC%AC%E0%AC%BF%E0%AC%B6%E0%AD%87%E0%AC%B7:%E0%AC%B8%E0%AC%AC%E0%AD%81%E0%AC%AA%E0%AD%83%E0%AC%B7%E0%AD%8D%E0%AC%A0%E0%AC%BE/%E0%AC%85",
        "te" => "https://te.wikipedia.org/wiki/%E0%B0%AA%E0%B1%8D%E0%B0%B0%E0%B0%A4%E0%B1%8D%E0%B0%AF%E0%B1%87%E0%B0%95:%E0%B0%85%E0%B0%A8%E0%B1%8D%E0%B0%A8%E0%B0%BF%E0%B0%AA%E0%B1%87%E0%B0%9C%E0%B1%80%E0%B0%B2%E0%B1%81?from=%E0%B0%85&to=&namespace=0",
        "gu" => "https://gu.wikipedia.org/wiki/%E0%AA%B5%E0%AA%BF%E0%AA%B6%E0%AB%87%E0%AA%B7:%E0%AA%AC%E0%AA%A7%E0%AA%BE%E0%AA%82%E0%AA%AA%E0%AA%BE%E0%AA%A8%E0%AA%BE%E0%AA%82/%E0%AA%85",
        "kn" => "https://kn.wikipedia.org/w/index.php?title=%E0%B2%B5%E0%B2%BF%E0%B2%B6%E0%B3%87%E0%B2%B7:AllPages&from=%E0%B2%85%E0%B2%82%E0%B2%9C%E0%B3%81%E0%B2%AE%E0%B3%8D+%E0%B2%9A%E0%B3%8B%E0%B2%AA%E0%B3%8D%E0%B2%B0",
        "mr" => "https://mr.wikipedia.org/w/index.php?title=%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:%E0%A4%B8%E0%A4%B0%E0%A5%8D%E0%A4%B5_%E0%A4%AA%E0%A4%BE%E0%A4%A8%E0%A5%87&from=%E0%A4%85%E0%A4%81%E0%A4%9F%E0%A5%8B%E0%A4%A8%E0%A5%80+%E0%A4%B5%E0%A5%8D%E0%A4%B9%E0%A4%BE%E0%A4%A8+%E0%A4%B2%E0%A5%80%E0%A4%B5%E0%A5%87%E0%A4%A8%E0%A4%B9%E0%A5%8B%E0%A4%95",
        "pi" => "https://pi.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B8%E0%A5%87%E0%A4%B8:AllPages?from=%E0%A4%85&to=&namespace=0",
        "sa" => "https://sa.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7%E0%A4%83:%E0%A4%B8%E0%A4%B0%E0%A5%8D%E0%A4%B5%E0%A4%AA%E0%A5%83%E0%A4%B7%E0%A5%8D%E0%A4%A0%E0%A4%BE%E0%A4%A8%E0%A4%BF?from=%E0%A4%85&to=&namespace=0",
        "ta" => "https://ta.wikipedia.org/wiki/%E0%AE%9A%E0%AE%BF%E0%AE%B1%E0%AE%AA%E0%AF%8D%E0%AE%AA%E0%AF%81:AllPages?from=%E0%AE%85&to=&namespace=0",
        "pa" => "https://pa.wikipedia.org/wiki/%E0%A8%96%E0%A8%BC%E0%A8%BE%E0%A8%B8:%E0%A8%B8%E0%A8%BE%E0%A8%B0%E0%A9%87_%E0%A8%B8%E0%A8%AB%E0%A8%BC%E0%A9%87?from=%E0%A8%85&to=&namespace=0",
        "en" => "https://en.wikipedia.org/wiki/Special:AllPages?from=A&to=&namespace=0",
        _ => return Err("Unsupported language".into()),
    };

    let client = Client::new();
    let mut links_count = 0;
    let mut batch_count = 0;

    let url = Url::parse(start_url)?;
    let main_url = url.host_str().ok_or("Invalid URL")?;

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

        if let Some(next_link) = parsed_html
            .select(&nav_selector)
            .nth(1)
            .and_then(|div| div.select(&a_selector).nth(1))
            .and_then(|a| a.value().attr("href"))
            .map(|link| format!("https://{}{}", main_url, link))
        {
            next_batch_link = next_link;
        } else {
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
        match fs::create_dir("wp_downloads") {
            Ok(()) => println!("Directory created successfully"),
            Err(err) => println!("Error creating directory: {}", err),
        }
        let dir_path = Path::new(".");
        let files: Vec<_> = fs::read_dir(dir_path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file()
                    && entry.path().extension().and_then(|s| s.to_str()) == Some("links")
            })
            .collect();
        let mut batch_count = 0;
        for each_file in files {
            let file_path = each_file.path();
            batch_count += 1;
            println!("\x1B[32mDownloading Batch No. {}\x1B[0m", batch_count);
            match download_from_file(file_path.to_str().unwrap()) {
                Some(_) => continue,
                None => continue,
            }
        }
    }

    println!("📊 Total batches done: {}", batch_count);
    println!("🔗 Total links saved: {}", links_count);

    Ok(())
}
