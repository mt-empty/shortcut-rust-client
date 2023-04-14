use anyhow::{Context, Result};
use clap::Parser;
use core::panic;
use reqwest::{blocking::Client, Proxy};
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Duration;
use zip::ZipArchive;

const PAGES_BASE_DIR: &str = "/opt/shortcut/pages/";
const PAGES_FILE_EXT: &str = "md";
const DOWNLOAD_ARCHIVE_URL: &str =
    "https://github.com/mt-empty/shortcut-pages/releases/latest/download/shortcuts.zip";

const ANSI_COLOUR_RESET_FG: &str = "\x1b[39m";
const ANSI_COLOUR_TITLE_FG: &str = "\x1b[39m";
const ANSI_COLOUR_EXPLANATION_FG: &str = "\x1b[37m";
const ANSI_COLOUR_DESCRIPTION_FG: &str = "\x1b[37m";
const ANSI_COLOUR_SHORTCUT_FG: &str = "\x1b[32m";
const ANSI_COLOUR_CATEGORY_FG: &str = "\x1b[37m";
const ANSI_BOLD_ON: &str = "\x1b[1m";
const ANSI_BOLD_OFF: &str = "\x1b[22m";

#[derive(Parser)]
#[clap(arg_required_else_help(true))]
#[clap(about = format!("A fast shortcut client, pages are located at {}", PAGES_BASE_DIR ), author, version)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The program name, e.g. `firefox`
    #[arg()]
    program_name: Option<String>,

    /// List all available shortcut pages in the cache
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    list: Option<bool>,

    /// Update the local cache
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    update: Option<bool>,

    /// Remove colour from the output
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    no_colour: Option<bool>,
}

fn main() {
    let cli = Cli::parse();

    let is_colour_on = !cli.no_colour.unwrap();
    if cli.update.unwrap() {
        match update() {
            Ok(_) => {}
            Err(error) => {
                eprintln!("Could not update shortcut cache: {:?}", error);
                exit(1)
            }
        }
    } else if cli.list.unwrap() {
        list_shortcuts();
    } else {
        match cli.program_name.map(|s| s.trim().to_owned()) {
            Some(program_name) if !program_name.is_empty() => {
                // let program_name = program_name.trim().to_owned();
                get_shortcut_page(&program_name, is_colour_on);
            }
            _ => {
                eprintln!("No valid argument specified");
                exit(1)
            }
        }
    }
}

fn has_write_permission(path: PathBuf) -> Result<bool, std::io::Error> {
    match fs::create_dir_all(path.to_owned()) {
        Ok(()) => {
            let metadata = fs::metadata(path)?;
            Ok(!metadata.permissions().readonly())
        }
        Err(error) => Err(error),
    }
}

fn update() -> Result<()> {
    match has_write_permission(PathBuf::from(PAGES_BASE_DIR)) {
        Ok(true) => {}
        Ok(false) => {
            eprintln!(
                "Writing to {} requires sudo privileges, please run with sudo",
                PAGES_BASE_DIR
            );
            exit(1)
        }
        Err(error) => {
            eprintln!("Could not read shortcut pages directory: {:?}", error);
            exit(1)
        }
    }

    let mut client_builder = Client::builder().timeout(Duration::from_secs(10));
    if let Ok(ref host) = env::var("HTTP_PROXY") {
        if let Ok(proxy) = Proxy::http(host) {
            client_builder = client_builder.proxy(proxy);
        }
    }
    if let Ok(ref host) = env::var("HTTPS_PROXY") {
        if let Ok(proxy) = Proxy::http(host) {
            client_builder = client_builder.proxy(proxy);
        }
    }
    let client = client_builder
        .build()
        .context("Could not instantiate HTTP client")?;
    let mut resp = client
        .get(DOWNLOAD_ARCHIVE_URL)
        .send()?
        .error_for_status()
        .with_context(|| {
            format!(
                "Could not download shortcut pages from {}",
                DOWNLOAD_ARCHIVE_URL,
            )
        })?;
    let mut buf: Vec<u8> = vec![];
    let _bytes_downloaded = resp.copy_to(&mut buf)?;

    let mut archive =
        ZipArchive::new(Cursor::new(buf)).context("Could not decompress downloaded ZIP archive")?;

    // Extract archive into pages dir
    archive
        .extract(PAGES_BASE_DIR)
        .context("Could not decompress downloaded ZIP archive")?;
    println!("Successfully updated cache");

    Ok(())
}

fn list_shortcuts() {
    if !Path::new(PAGES_BASE_DIR).exists() {
        eprintln!("Shortcut pages directory does not exist, please run shortcut --update");
        exit(1)
    }
    let dir = Path::new(PAGES_BASE_DIR);
    if dir.is_dir() {
        let mut entries = fs::read_dir(dir).unwrap();
        if entries.next().is_none() {
            eprintln!("Shortcut pages directory is empty, please run shortcut --update");
            exit(1)
        }

        for entry in entries {
            let file_name = entry.unwrap().file_name().to_string_lossy().to_string();
            let path_buf = PathBuf::from(file_name);
            let file_stem = path_buf.with_extension("");
            println!("{}", file_stem.display())
        }
    }
}

fn get_shortcut_page(program_name: &String, is_colour_on: bool) -> bool {
    if !Path::new(PAGES_BASE_DIR).exists() || !Path::new(PAGES_BASE_DIR).is_dir() {
        eprintln!("Shortcut pages directory does not exist, please run shortcut --update");
        exit(1)
    } else if fs::read_dir(PAGES_BASE_DIR).unwrap().next().is_none() {
        eprintln!("Shortcut pages directory is empty, please run shortcut --update");
        exit(1)
    }

    let clean_program_name = Path::new(program_name).file_name();
    match clean_program_name {
        Some(name) => {
            let mut program_path: PathBuf = PathBuf::from(PAGES_BASE_DIR);
            program_path.push(name.to_ascii_lowercase());
            program_path.set_extension(PAGES_FILE_EXT);

            match File::open(program_path) {
                Ok(file_descriptor) => return parse_shortcut_page(file_descriptor, is_colour_on),
                Err(_error) => {
                    eprintln!("No page available for \"{}\"", program_name);
                    return false;
                }
            };
        }
        None => {
            eprintln!("Invalid program name \"{}\"", program_name);
            exit(1)
        }
    };
}

fn parse_shortcut_page(file_descriptor: File, is_colour_on: bool) -> bool {
    let reader = BufReader::new(file_descriptor);

    for line_results in reader.lines() {
        let line = match line_results {
            Ok(file) => file,
            Err(error) => panic!("Problem reading the file: {:?}", error),
        };

        let mut start = false;
        for (i, c) in line.char_indices() {
            if !start {
                match c {
                    '#' => {
                        if is_colour_on {
                            print!("{}", ANSI_BOLD_ON);
                            print!("{}", ANSI_COLOUR_TITLE_FG);
                        }
                        start = true;
                    }
                    '$' => {
                        if is_colour_on {
                            print!("{}", ANSI_COLOUR_CATEGORY_FG);
                        }
                        start = true;
                    }
                    '>' => {
                        if is_colour_on {
                            print!("{}", ANSI_COLOUR_EXPLANATION_FG);
                        }
                        start = true;
                    }
                    '`' => {
                        if is_colour_on {
                            print!("{}", ANSI_COLOUR_SHORTCUT_FG);
                        }
                        start = true;
                    }
                    _ => {
                        print!("{}", c);
                    }
                };
            } else {
                match c {
                    '{' => {
                        let line_bytes = line.as_bytes();
                        if line_bytes[i + 1] as char == '{' {
                            if is_colour_on {
                                print!("{}", ANSI_COLOUR_DESCRIPTION_FG);
                            }
                        } else if line_bytes[i - 1] as char == '{' {
                            continue;
                        } else {
                            print!("{}", c);
                        }
                    }
                    '}' => {
                        let line_bytes = line.as_bytes();
                        if line_bytes[i + 1] as char == '}' {
                            if is_colour_on {
                                print!("{}", ANSI_COLOUR_DESCRIPTION_FG);
                            }
                        } else if line_bytes[i - 1] as char == '}' {
                            continue;
                        } else {
                            print!("{}", c);
                        }
                    }
                    _ => {
                        print!("{}", c);
                    }
                };
            }
        }
        println!("{}{}", ANSI_BOLD_OFF, ANSI_COLOUR_RESET_FG);
    }
    return true;
}
