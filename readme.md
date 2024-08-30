# wp: wikipedia tools on your terminal

This projects aims to be the ultimate tool to view and download wikipedia articles on from your terminal. It also doubles as a tool for bulk downloading wikipedia articles as plaintext from a file listing all the desired links.

## Features

-  Print wikipedia articles on `stdout` as plaintext.
-  Save wikipedia articles to disk as plain text.
- Bulk download all of wikipedia for any particular language.

Currently supported languages for bulk download are: 

| Languages | Codes |
|-----------|------|
|Assamese | `as` |
| Bengali | `bn` |
| Bhojpuri | `bh` |
| Gujarati | `gu` |
| Hindi | `hi` |
| Kannada | `kn` |
| Marathi | `mr` |
| Nepali | `ne` |
| Oriya | `or` |
| Pali | `pi` |
| Punjabi | `pn` |
|Sanskrit | `sa` |
| Tamil | `ta` |
| Telugu | `te` | 

Raise an issue if you would like support for any other language.

## How to download `wp`

### For `bash`, `zsh` or similar
```
sudo curl -L https://github.com/amkhrjee/wp/releases/latest/download/wp -o /usr/local/bin/wp && sudo chmod +x /usr/local/bin/wp 
```
If you don't have `sudo`, simply download to the directory you have access to:

```
curl -L https://github.com/amkhrjee/wp/releases/latest/download/wp -o wp && chmod +x wp 
```

### For Windows Powershell (both legacy and the new `pwsh`)
```
Invoke-WebRequest -Uri https://github.com/amkhrjee/wp/releases/latest/download/wp.exe -OutFile wp.exe;
```

## How to use `wp` to parse wikipedia articles

```
wp --link <LINK_TO_THE_ARTICLE OR THE FILE NAME> [--save] 
```

The `--save` flag saves the article to disk rather than outputting to stdout.

For bulk downloading from multiple links, create a file with one link per line.

When bulk downloading, the `--save` flag is automatically added.


## Scraping wikipedia

If you want to scrape *all* of wikipedia into plain text files for any particular language, paste the following script in your terminal:

```
wp --lang <LANGUAGE_CODE> [--links-only]
```
Setting the `--links-only` flag will only save the links aggregated into a zip file, without downloading the actual contents.

On Windows, this should be
```
.\wp.exe --lang <LANGUAGE_CODE> [--links-only]
```

## Future goals

- Exportability in Markdown and RST format.
- Display articles with a TUI (was available till commit [`5a3b`](https://github.com/amkhrjee/wp/tree/5a3b0c3b85e46fa6cd933af5d3ea36b3ac1d1a0d)).
- Release as a crate on [crates.io](https://crates.io)
- Distribute via package managers

This is a *very* alpha software, so use at your own risk (. ❛ ᴗ ❛.)

PRs are welcome. 