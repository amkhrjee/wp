import argparse
import os
from urllib.parse import urlparse
from zipfile import ZIP_DEFLATED, ZipFile

import requests
from bs4 import BeautifulSoup
from tqdm import tqdm


def get_links(parsed_html, main_url, batch_count):
    print("⚡ Starting batch", batch_count)
    print("⚡ Scraping links...")

    # Note: the links are in the form /wiki/title
    links = [
        a["href"]
        # for
        for uls in parsed_html.find_all("ul", attrs={"class": "mw-allpages-chunk"})
        for li in uls.find_all("li", attrs={"class": ""})
        for a in li.find_all("a", href=True)
    ]
    links = ["https://" + main_url + link for link in links]

    batch_size = len(links)
    print(f"⚡ Accumualated {batch_size} links")

    with open(f"{args.lang}_{batch_count}.links", "w") as f:
        if len(links) > 2:
            for link in links[:-1]:
                f.write(link + "\n")
        if len(links) == 1:
            f.write(links.pop())
    return batch_size


parser = argparse.ArgumentParser(
    description="Scrape all wikipedia links for any language."
)

parser.add_argument(
    "--lang",
    required=True,
    choices=["as", "hi", "bn"],
    help="Choose the language for the wikipedia links.",
)

args = parser.parse_args()
start_url = ""

if args.lang == "as":
    start_url = "https://as.wikipedia.org/wiki/%E0%A6%AC%E0%A6%BF%E0%A6%B6%E0%A7%87%E0%A6%B7:%E0%A6%B8%E0%A6%95%E0%A6%B2%E0%A7%8B%E0%A6%AC%E0%A7%8B%E0%A7%B0_%E0%A6%AA%E0%A7%83%E0%A6%B7%E0%A7%8D%E0%A6%A0%E0%A6%BE/%E0%A6%85"
elif args.lang == "bn":
    start_url = "https://bn.wikipedia.org/wiki/%E0%A6%AC%E0%A6%BF%E0%A6%B6%E0%A7%87%E0%A6%B7:%E0%A6%B8%E0%A6%AC_%E0%A6%AA%E0%A6%BE%E0%A6%A4%E0%A6%BE/%E0%A6%85"
elif args.lang == "hi":
    start_url = "https://hi.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:%E0%A4%B8%E0%A4%AD%E0%A5%80_%E0%A4%AA%E0%A5%83%E0%A4%B7%E0%A5%8D%E0%A4%A0/%E0%A4%85"
else:
    exit(1)

# Some stats
links_count = 0
batch_count = 1

# Starting point
main_url = urlparse(start_url).netloc
response = requests.get(start_url)
html = response.text
parsed_html = BeautifulSoup(html, "html.parser")
print("💭 Links will be saved to your current directory as zip.")

print("")
links_count += get_links(parsed_html, main_url, batch_count)
print("⚡ Starting next batch...")
batch_count += 1

# Next batch
next_batch_link = (
    parsed_html.find_all("div", attrs={"class": "mw-allpages-nav"})[1]
).find_all("a", href=True)[1]["href"]

while len(next_batch_link) != 0:
    next_batch_link = "https://" + main_url + next_batch_link
    main_url = urlparse(next_batch_link).netloc
    response = requests.get(next_batch_link)
    html = response.text
    parsed_html = BeautifulSoup(html, "html.parser")

    batch_count += 1
    links_count += get_links(parsed_html, main_url, batch_count)

    try:
        next_batch_link = (
            parsed_html.find_all("div", attrs={"class": "mw-allpages-nav"})[1]
        ).find_all("a", href=True)[1]["href"]
    except:  # noqa: E722
        break

print("✅ All links saved.")

# Save to zip
print("🗃️ Zipping up all the links...")
with tqdm(total=batch_count) as pb:
    with ZipFile(f"{args.lang}.zip", "w", ZIP_DEFLATED) as zip_file:
        for file in os.listdir("./"):
            if file.endswith(".links"):
                file_path = os.path.join("./", file)
                zip_file.write(file_path, file)
                os.remove(file_path)
                pb.update(1)

print("📊 Total batches done: ", batch_count)
print("🔗 Total links saved: ", links_count)
