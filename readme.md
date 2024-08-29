# wp: wikipedia tools on your terminal

This projects aims to be the ultimate tool to view and download wikipedia articles on from your terminal. It also doubles as a tool for bulk downloading wikipedia articles as plaintext from a file listing all the desired links.

## Features

-  Display wikipedia articles on `stdout` as plaintext.
-  Save wikipedia articles to disk as plaintext.

### How to use

```
wp --link <LINK_TO_THE_ARTICLE OR THE FILE NAME> [--save]
```

The `--save` flag saves the article to disk rather than outputting to stdout.

For bulk downloading from multiple links, create a file with one link per line.

When bulk downloading, the `--save` flag is automatically added.

### How to get `wp`

Checkout the Releases page and download the binary from the latest release for your machine. Only 64-bit binaries for Windows and Linux systems are available for now. Feel free to build from source for your architecture with `cargo build --release`.

## Future goals

- Exportability in Markdown and RST format.
- Display articles with a TUI (was available till commit [`5a3b`](https://github.com/amkhrjee/wp/tree/5a3b0c3b85e46fa6cd933af5d3ea36b3ac1d1a0d)).
- In-built scraper for bulk downloading all wikipedia articles for any given language.

This is a *very* alpha software, so use at your own risk (. ❛ ᴗ ❛.)

PRs are welcome. 