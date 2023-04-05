// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use atty::Stream;
use std::{io::{self, Read}, process::exit};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

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

    // if stdout is not redirected, print with syntax highlighting
    if atty::is(Stream::Stdout) {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = match is_json {
            true => ps.find_syntax_by_extension("json").unwrap(),
            false => ps.find_syntax_by_extension("yaml").unwrap(),
        };

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        for line in LinesWithEndings::from(output.as_str()) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            print!("{}", escaped);
        }
    } else {
        println!("{}", output);
    }

    exit(EXIT_SUCCESS);
}
