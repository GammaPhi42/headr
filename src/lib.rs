// use assert_cmd::assert;
use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3 is an OK integer
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Any string is an error
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // A zero is an error
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("Ken Youens-Clark <kyclark@gmail.com>")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("files")
                .help("Input files")
                .multiple(true)
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .value_name("LINES")
                .default_value("10")
                .help("print the first lines of each file"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .takes_value(true)
                .value_name("BYTES")
                .conflicts_with("lines")
                .help("print the first BYTES of each file"),
        )
        .get_matches();

    let bytes: Option<usize>;
    let lines: usize;
    if let Some(num_bytes) = matches.value_of("bytes") {
        lines = 0;
        match parse_positive_int(num_bytes) {
            Ok(parse_ok) => bytes = Some(parse_ok),
            Err(parse_err) => {
                eprint!("illegal byte count -- {}", parse_err);
                std::process::exit(1);
            }
        }
    } else {
        bytes = None;
        if let Some(num_lines) = matches.value_of("lines") {
            match parse_positive_int(num_lines) {
                Ok(parse_ok) => lines = parse_ok,
                Err(parse_err) => {
                    eprint!("illegal line count -- {}", parse_err);
                    std::process::exit(1);
                }
            }
        } else {
            lines = 0;
        }
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut print_line_break = false;
    let num_files = config.files.len();
    for filename in config.files {
        match config.bytes {
            None => {
                let num_lines = config.lines;

                match open(&filename) {
                    Err(err) => {
                        eprintln!("{}: {}", filename, err);
                        continue;
                    }
                    Ok(mut file_reader) => {
                        if num_files > 1 {
                            if print_line_break {
                                println!("\n==> {} <==", filename);
                            } else {
                                println!("==> {} <==", filename);
                            }
                        }
                        let mut lines_printed = 0;
                        while lines_printed < num_lines {
                            let mut file_line = String::new();
                            let file_line_bytes = file_reader.read_line(&mut file_line);
                            match file_line_bytes {
                                Err(_) => break,
                                Ok(0) => break,
                                Ok(_) => print!("{}", file_line),
                            }
                            lines_printed += 1;
                        }
                    }
                }
            }
            Some(num_bytes) => match open(&filename) {
                Err(err) => {
                    eprintln!("{}: {}", filename, err);
                    continue;
                }

                Ok(mut file_buf) => {
                    if num_files > 1 {
                        if print_line_break {
                            println!("\n==> {} <==", filename);
                        } else {
                            println!("==> {} <==", filename);
                        }
                    }

                    let mut file_bytes = Vec::new();
                    let bytes_in_file = file_buf.read_to_end(&mut file_bytes);
                    match bytes_in_file {
                        Err(_) => break,
                        Ok(_bytes_in_file) => {
                            file_bytes.truncate(num_bytes);
                            print!("{}", String::from_utf8_lossy(file_bytes.as_slice()));
                        }
                    }
                }
            },
        }
        if !print_line_break {
            print_line_break = true;
        }
    }

    Ok(())
}
