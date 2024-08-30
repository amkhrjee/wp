# wp: wikipedia tools on your terminal

This projects aims to be the ultimate tool to view and download wikipedia articles on from your terminal. It also doubles as a tool for bulk downloading wikipedia articles as plaintext from a file listing all the desired links.

## Features

-  Print wikipedia articles on `stdout` as plaintext.
-  Save wikipedia articles to disk as plain text.
- Bulk download all of wikipedia for any particular language.

Currently supported languages for bulk download are: 
<div style="display: flex; gap: 15px;">
<div>

| Languages | Codes |
|-----------|------|
|Assamese | `as` |
| Bengali | `bn` |
| Bhojpuri | `bh` |
| Gujarati | `gu` |
| Hindi | `hi` |
| Kannada | `kn` |
| Marathi | `mr` |

</div>

<div>

| Languages | Codes |
|-----------|------|
| Nepali | `ne` |
| Oriya | `or` |
| Pali | `pi` |
| Punjabi | `pn` |
|Sanskrit | `sa` |
| Tamil | `ta` |
| Telugu | `te` | 

</div>
</div>


Raise an issue if you would like support for any other language.

## How to use `wp`

```
wp --link <LINK_TO_THE_ARTICLE OR THE FILE NAME> [--save]
```

The `--save` flag saves the article to disk rather than outputting to stdout.

For bulk downloading from multiple links, create a file with one link per line.

When bulk downloading, the `--save` flag is automatically added.

## How to get `wp`

Checkout the Releases page and download the binary from the latest release for your machine. Only 64-bit binaries for Windows and Linux systems are available for now. Feel free to build from source for your architecture with `cargo build --release`.

## Scraping wikipedia

If you want to scrape *all* of wikipedia into plain text files for any particular language, paste the following script in your terminal:

### For `bash`, `zsh` or similar
```
curl -L https://github.com/amkhrjee/wp/releases/latest/download/wp -o wp && chmod +x wp && curl -L https://github.com/amkhrjee/wp/releases/latest/download/downloader -o downloader && chmod +x downloader
```
### For Windows Powershell (both legacy and new `pwsh`)
```
Invoke-WebRequest -Uri https://github.com/amkhrjee/wp/releases/latest/download/wp.exe -OutFile wp.exe;
Invoke-WebRequest -Uri https://github.com/amkhrjee/wp/releases/latest/download/downloader.exe -OutFile downloader.exe;
```

Next,

```
./downloader --lang <YOUR_CHOICE> --save
```
If you're on  Windows, do this instead:

```
.\downloader.exe --lang <YOUR_CHOICE> --save
```

> **Note:** Without the `--save` flag, the `downloader` will ony aggregate the links to articles in a zip file.

## Future goals

- Exportability in Markdown and RST format.
- Display articles with a TUI (was available till commit [`5a3b`](https://github.com/amkhrjee/wp/tree/5a3b0c3b85e46fa6cd933af5d3ea36b3ac1d1a0d)).
- Release as a crate on [crates.io](https://crates.io)
- Distribute via package managers



This is a *very* alpha software, so use at your own risk (. ❛ ᴗ ❛.)

PRs are welcome. 