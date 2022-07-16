// use assert_cmd::assert;
use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

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
    let num_lines = config.lines;
    let num_bytes = config.bytes;
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(mut file_reader) => match num_bytes {
                Some(num_bytes) => {
                    let mut buffer: Vec<u8> = Vec::with_capacity(num_bytes);
                    buffer = file_reader.fill_buf().unwrap().to_vec();
                    file_reader.consume(num_bytes);
                    if !buffer.is_empty() {
                        println!("==> {} <==", filename);
                    }
                    for x in buffer {
                        print!("{}", x);
                    }
                }
                None => {
                    for _line in 0..num_lines {
                        let mut line_to_print = String::new();
                        match file_reader.read_line(&mut line_to_print) {
                            Ok(0) => break,
                            Ok(line_to_print) => {
                                println!("==> {} <==", filename);
                                println!("{}", line_to_print)
                            }
                            Err(_) => std::process::exit(1),
                        }
                    }
                }
            },
        }
    }
    Ok(())
}
