use args::Arguments;
use atty::Stream;
use clap::{Parser};
use std::{io::{self, Read}, process::exit};

mod args;
mod win1;

const EXIT_SUCCESS: i32 = 0;

fn main() {
    let args = Arguments::parse();

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

    match args.subcommand {
        args::SubCommand::Query { key_path, value_name, recurse } => {
            println!("Get key_path: {}, value_name: {:?}, recurse: {}", key_path, value_name, recurse);
        },
        args::SubCommand::Set { key_path, value } => {
            println!("Set key_path: {}, value: {}", key_path, value);
        },
        args::SubCommand::Test => {
            println!("Test");
        },
        args::SubCommand::Remove { key_path, value_name, recurse } => {
            println!("Remove key_path: {}, value_name: {:?}, recurse: {}", key_path, value_name, recurse);
        },
        args::SubCommand::Find { key_path, find, recurse, keys_only, values_only } => {
            println!("Find key_path: {}, find: {}, recurse: {:?}, keys_only: {:?}, values_only: {:?}", key_path, find, recurse, keys_only, values_only);
        },
    }

    println!("stdin: {:?}", stdin);
    exit(EXIT_SUCCESS);
}
