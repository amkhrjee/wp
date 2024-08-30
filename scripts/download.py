import argparse
import os
from urllib.parse import urlparse
from zipfile import ZIP_DEFLATED, ZipFile

import requests
from bs4 import BeautifulSoup
from tqdm import tqdm


def get_links(parsed_html, main_url, batch_count):
    print("⚡ Starting batch", batch_count)

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
    choices=[
        "as",
        "hi",
        "bn",
        "bh",
        "ne",
        "or",
        "te",
        "gu",
        "kn",
        "mr",
        "pi",
        "sa",
        "ta",
    ],
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
elif args.lang == "bh":
    start_url = "https://bh.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:%E0%A4%B8%E0%A4%BE%E0%A4%B0%E0%A4%BE_%E0%A4%AA%E0%A4%A8%E0%A5%8D%E0%A4%A8%E0%A4%BE?from=%E0%A4%85&to=&namespace=0"
elif args.lang == "ne":
    start_url = "https://ne.wikipedia.org/w/index.php?title=%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:AllPages&from=%E0%A4%85%E0%A4%A4%E0%A4%BF%E0%A4%AF%E0%A4%A5%E0%A4%BE%E0%A4%B0%E0%A5%8D%E0%A4%A5%E0%A4%B5%E0%A4%BE%E0%A4%A6"
elif args.lang == "or":
    start_url = "https://or.wikipedia.org/wiki/%E0%AC%AC%E0%AC%BF%E0%AC%B6%E0%AD%87%E0%AC%B7:%E0%AC%B8%E0%AC%AC%E0%AD%81%E0%AC%AA%E0%AD%83%E0%AC%B7%E0%AD%8D%E0%AC%A0%E0%AC%BE/%E0%AC%85"
elif args.lang == "te":
    start_url = "https://te.wikipedia.org/wiki/%E0%B0%AA%E0%B1%8D%E0%B0%B0%E0%B0%A4%E0%B1%8D%E0%B0%AF%E0%B1%87%E0%B0%95:%E0%B0%85%E0%B0%A8%E0%B1%8D%E0%B0%A8%E0%B0%BF%E0%B0%AA%E0%B1%87%E0%B0%9C%E0%B1%80%E0%B0%B2%E0%B1%81?from=%E0%B0%85&to=&namespace=0"
elif args.lang == "gu":
    start_url = "https://gu.wikipedia.org/wiki/%E0%AA%B5%E0%AA%BF%E0%AA%B6%E0%AB%87%E0%AA%B7:%E0%AA%AC%E0%AA%A7%E0%AA%BE%E0%AA%82%E0%AA%AA%E0%AA%BE%E0%AA%A8%E0%AA%BE%E0%AA%82/%E0%AA%85"
elif args.lang == "kn":
    start_url = "https://kn.wikipedia.org/w/index.php?title=%E0%B2%B5%E0%B2%BF%E0%B2%B6%E0%B3%87%E0%B2%B7:AllPages&from=%E0%B2%85%E0%B2%82%E0%B2%9C%E0%B3%81%E0%B2%AE%E0%B3%8D+%E0%B2%9A%E0%B3%8B%E0%B2%AA%E0%B3%8D%E0%B2%B0"
elif args.lang == "mr":
    start_url = "https://mr.wikipedia.org/w/index.php?title=%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7:%E0%A4%B8%E0%A4%B0%E0%A5%8D%E0%A4%B5_%E0%A4%AA%E0%A4%BE%E0%A4%A8%E0%A5%87&from=%E0%A4%85%E0%A4%81%E0%A4%9F%E0%A5%8B%E0%A4%A8%E0%A5%80+%E0%A4%B5%E0%A5%8D%E0%A4%B9%E0%A4%BE%E0%A4%A8+%E0%A4%B2%E0%A5%80%E0%A4%B5%E0%A5%87%E0%A4%A8%E0%A4%B9%E0%A5%8B%E0%A4%95"
elif args.lang == "pi":
    start_url = "https://pi.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B8%E0%A5%87%E0%A4%B8:AllPages?from=%E0%A4%85&to=&namespace=0"
elif args.lang == "sa":
    start_url = "https://sa.wikipedia.org/wiki/%E0%A4%B5%E0%A4%BF%E0%A4%B6%E0%A5%87%E0%A4%B7%E0%A4%83:%E0%A4%B8%E0%A4%B0%E0%A5%8D%E0%A4%B5%E0%A4%AA%E0%A5%83%E0%A4%B7%E0%A5%8D%E0%A4%A0%E0%A4%BE%E0%A4%A8%E0%A4%BF?from=%E0%A4%85&to=&namespace=0"
elif args.lang == "ta":
    start_url = "https://ta.wikipedia.org/wiki/%E0%AE%9A%E0%AE%BF%E0%AE%B1%E0%AE%AA%E0%AF%8D%E0%AE%AA%E0%AF%81:AllPages?from=%E0%AE%85&to=&namespace=0"
elif args.lang == "pa":
    start_url = "https://pa.wikipedia.org/wiki/%E0%A8%96%E0%A8%BC%E0%A8%BE%E0%A8%B8:%E0%A8%B8%E0%A8%BE%E0%A8%B0%E0%A9%87_%E0%A8%B8%E0%A8%AB%E0%A8%BC%E0%A9%87?from=%E0%A8%85&to=&namespace=0"
else:
    exit(1)

# Some stats
links_count = 0
batch_count = 0

# Starting point
main_url = urlparse(start_url).netloc
response = requests.get(start_url)
html = response.text
parsed_html = BeautifulSoup(html, "html.parser")
print("💭 Links will be saved to your current directory as zip.")
print("⚡ Scraping links...")


print("")
batch_count += 1
links_count += get_links(parsed_html, main_url, batch_count)

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
