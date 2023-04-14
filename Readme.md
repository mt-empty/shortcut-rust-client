# Shortcut

A command line client for [shortcut-pages](https://github.com/mt-empty/shortcut-pages), written in Rust.

![](https://github.com/mt-empty/shortcut-c-client/blob/master/shortcut.gif)


## Installing

Install from source:
```bash
sudo curl -sSL https://github.com/mt-empty/shortcut-rust-client/releases/latest/download/shortcut -o /usr/local/bin/shortcut && \
  sudo chmod +x /usr/local/bin/shortcut && \
  sudo /usr/local/bin/shortcut --update 
```


## Usage

```
A fast shortcut client, pages are located at /tmp/shortcut/pages/

Usage: shortcut [OPTIONS] <PROGRAM_NAME>

Arguments:
  <PROGRAM_NAME>  The program name, e.g. `firefox`

Options:
  -l, --list       List all available shortcut pages in the cache
  -u, --update     Update the local cache
  -n, --no-colour  Remove colour from the output
  -h, --help       Print help information
  -V, --version    Print version information
```


## Contributing

Contributions are most welcome!

Bugs: open an issue here.

New features: open an issue here or feel free to send a pull request with the included feature.
