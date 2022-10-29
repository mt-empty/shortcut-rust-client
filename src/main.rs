use clap::Parser;
use std::fs::{self, File};
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

const PAGES_BASE_DIR: &str = "/opt/shortcut/pages/";
const PAGES_FILE_EXT: &str = ".md";

const ANSI_COLOUR_RESET_FG: &str = "\x1b[39m";
const ANSI_COLOUR_TITLE_FG: &str = "\x1b[39m";
const ANSI_COLOUR_EXPLANATION_FG: &str = "\x1b[37m";
const ANSI_COLOUR_DESCRIPTION_FG: &str = "\x1b[37m";
const ANSI_COLOUR_SHORTCUT_FG: &str = "\x1b[32m";
const ANSI_COLOUR_CATEGORY_FG: &str = "\x1b[37m";
const ANSI_BOLD_ON: &str = "\x1b[1m";
const ANSI_BOLD_OFF: &str = "\x1b[22m";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// program name
    program_name: String,

    /// List all available shortcut pages in the cache
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    list: Option<bool>,

    /// Update the local cache
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    update: Option<bool>,

    /// Remove colours from the output
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    no_colour: Option<bool>,
}

fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);

    //TODO, validate program_name
    let program_name = cli.program_name;
    let is_colour_on = !cli.no_colour.unwrap();

    if cli.list.unwrap() {
        list_shortcuts();
    } else {
        get_shortcut_page(&program_name, is_colour_on);
    }
}

fn list_shortcuts() {
    let paths = fs::read_dir(PAGES_BASE_DIR).unwrap();

    for path in paths {
        println!(
            "{}",
            path.unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .strip_suffix(".md")
                .unwrap()
        )
    }
}

fn get_shortcut_page(program_name: &String, is_colour_on: bool) -> bool {
    let mut program_path: PathBuf = PathBuf::from(PAGES_BASE_DIR);
    program_path.push(program_name.to_owned() + PAGES_FILE_EXT);

    match File::open(program_path) {
        Ok(file_descriptor) => return parse_shortcut_page(file_descriptor, is_colour_on),
        Err(_error) => {
            eprintln!("No page available for \"{}\"", program_name);
            return false;
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
        print!("{}", ANSI_BOLD_OFF);
        print!("{}", ANSI_COLOUR_RESET_FG);
        println!();
    }
    return true;
}
