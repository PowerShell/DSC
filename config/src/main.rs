use args::*;
use atty::Stream;
use clap::Parser;
use std::io::{self, Read};
use std::process::exit;
use dsc_lib::DscManager;

pub mod args;

fn main() {
    let args = Args::parse();

    let stdin: Option<String> = if atty::is(Stream::Stdin) {
        None
    } else {
        let mut buffer: Vec<u8> = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();
        let input = match String::from_utf8(buffer) {
            Ok(input) => input,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        Some(input)
    };

    let dsc = match DscManager::new() {
        Ok(dsc) => dsc,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    };

    match args.subcommand {
        SubCommand::List { resource_name } => {
            for resource in dsc.find_resource(&resource_name.unwrap_or_default()) {
                // convert to json
                let json = match serde_json::to_string(&resource) {
                    Ok(json) => json,
                    Err(err) => {
                        eprintln!("Error: {}", err);
                        exit(1);
                    }
                };
                println!("{}", json);
            }
        }
        SubCommand::Get { resource_name } => {
            println!("Get {}: {}", resource_name, stdin.unwrap_or_default());
        }
        SubCommand::Set { resource_name } => {
            println!("Set {}: {}", resource_name, stdin.unwrap_or_default());
        }
        SubCommand::Test { resource_name } => {
            println!("Test {}: {}", resource_name, stdin.unwrap_or_default());
        }
        SubCommand::Flush => {
            println!("Flush");
        }
    }
}
