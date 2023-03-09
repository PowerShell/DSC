use atty::Stream;
use std::{io::{self, Read}, process::exit};

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_INPUT: i32 = 1;

fn main() {
    let input: String = if atty::is(Stream::Stdin) {
        eprintln!("Error: Input JSON/YAML via STDIN is required.");
        exit(EXIT_INVALID_INPUT);
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {}", e);
                exit(EXIT_INVALID_INPUT);
            }
        }
    };

    let mut is_json = true;
    let input: serde_json::Value = match serde_json::from_str(&input) {
        Ok(json) => json,
        Err(_) => {
            is_json = false;
            match serde_yaml::from_str(&input) {
                Ok(yaml) => yaml,
                Err(err) => {
                    eprintln!("Error: Input is not valid JSON or YAML: {}", err);
                    exit(EXIT_INVALID_INPUT);
                }
            }
        }
    };

    let output = match is_json {
        true => serde_yaml::to_string(&input).unwrap(),
        false => serde_json::to_string_pretty(&input).unwrap(),
    };

    println!("{}", output);
    exit(EXIT_SUCCESS);
}
