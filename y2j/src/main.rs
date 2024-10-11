// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{io::{self, Read, IsTerminal}, process::exit};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

const EXIT_SUCCESS: i32 = 0;
const EXIT_INVALID_INPUT: i32 = 1;

fn main() {
    let input: String = if std::io::stdin().is_terminal() {
        eprintln!("Error: Input JSON/YAML via STDIN is required.");
        exit(EXIT_INVALID_INPUT);
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {e}");
                exit(EXIT_INVALID_INPUT);
            }
        }
    };

    let mut is_json = true;
    let input: serde_json::Value = if let Ok(json) = serde_json::from_str(&input) { json } else {
        is_json = false;
        match serde_yaml::from_str(&input) {
            Ok(yaml) => yaml,
            Err(err) => {
                eprintln!("Error: Input is not valid JSON or YAML: {err}");
                exit(EXIT_INVALID_INPUT);
            }
        }
    };

    let output = if is_json {
        serde_yaml::to_string(&input).unwrap()
    } else {
        serde_json::to_string_pretty(&input).unwrap()
    };

    // if stdout is not redirected, print with syntax highlighting
    if std::io::stdin().is_terminal() {
        let ps = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let syntax = if is_json {
            ps.find_syntax_by_extension("json").unwrap()
        } else {
            ps.find_syntax_by_extension("yaml").unwrap()
        };

        let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);

        for line in LinesWithEndings::from(output.as_str()) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            print!("{escaped}");
        }
    } else {
        println!("{output}");
    }

    exit(EXIT_SUCCESS);
}
