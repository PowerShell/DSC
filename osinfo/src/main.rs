// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod config;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let include_name = if args.len() > 1 && args[1] == "export" {
        true
    } else {
        false
    };
    let json = serde_json::to_string(&config::OsInfo::new(include_name)).unwrap();
    println!("{json}");
}
