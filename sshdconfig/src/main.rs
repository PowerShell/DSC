use args::*;
use atty::Stream;
use clap::Parser;
use std::io::{self, Read};

pub mod args;
pub mod config;
pub mod match_data;
pub mod sshdconfig_error;

fn main() {
    let args = Cli::parse();

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

    match args.command {
        Commands::Get { input_config_path, input_config_json, curr_config_path } => {
            println!("Get args: -f: {} -j: {} -p: {} stdin: {}", 
            input_config_path.unwrap_or_default(), 
            input_config_json.unwrap_or_default(), 
            curr_config_path.unwrap_or_default(), 
            stdin.unwrap_or_default());
        }
        Commands::Set { input_config_path, input_config_json, curr_config_path } => {
            println!("Set args: -f: {} -j: {} -p: {} stdin: {}", 
            input_config_path.unwrap_or_default(), 
            input_config_json.unwrap_or_default(), 
            curr_config_path.unwrap_or_default(), 
            stdin.unwrap_or_default());
        }
        Commands::Test { input_config_path, input_config_json, curr_config_path } => {
            println!("Test args: -f: {} -j: {} -p: {} stdin: {}", 
            input_config_path.unwrap_or_default(), 
            input_config_json.unwrap_or_default(), 
            curr_config_path.unwrap_or_default(), 
            stdin.unwrap_or_default());
        }
    }
}
