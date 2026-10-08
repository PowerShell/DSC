// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{io::{self, Read, IsTerminal}, process::ExitCode};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

const EXIT_INVALID_INPUT: u8 = 1;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(EXIT_INVALID_INPUT)
        }
    }
}

fn run() -> Result<(), String> {
    let input: String = if std::io::stdin().is_terminal() {
        return Err("Error: Input JSON/YAML via STDIN is required.".to_string());
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        String::from_utf8(buffer).map_err(|e| format!("Invalid UTF-8 sequence: {e}"))?
    };

    let mut is_json = true;
    let input: serde_json::Value = if let Ok(json) = serde_json::from_str(&input) { json } else {
        is_json = false;
        match serde_yaml::from_str(&input) {
            Ok(yaml) => yaml,
            Err(err) => return Err(format!("Error: Input is not valid JSON or YAML: {err}")),
        }
    };

    let output = if is_json {
        serde_yaml::to_string(&input).unwrap()
    } else {
        serde_json::to_string_pretty(&input).unwrap()
    };

    // if stdout is not redirected, print with syntax highlighting
    if std::io::stdout().is_terminal() {
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

    Ok(())
}
