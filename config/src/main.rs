use args::*;
use atty::Stream;
use clap::Parser;
use std::io::{self, Read};

pub mod args;
pub mod discovery;
pub mod dscresources;
pub mod dscerror;

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

    match args.subcommand {
        SubCommand::List { resource_name } => {
            let discovery = discovery::Discovery::new();
            for resource in discovery.find_resource(&resource_name.unwrap_or_default()) {
                println!("{} = {:?}", resource.name, resource.implemented_as);
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
