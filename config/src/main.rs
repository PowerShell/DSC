use args::*;
use atty::Stream;
use std::{env, io};

pub mod args;
pub mod discovery;
pub mod dscresources;
pub mod dscerror;

pub const EXIT_INVALID_ARGS: i32 = 1;
pub const EXIT_INVALID_INPUT: i32 = 2;

fn main() {
    let args = Args::new(&mut env::args(), &mut io::stdin(), atty::is(Stream::Stdin));
    match args.subcommand {
        SubCommand::List => {
            // perform discovery
            println!("List {}", args.filter);
        }
        SubCommand::Get => {
            // perform discovery
            println!("Get {}: {}", args.resource, args.stdin);
        }
        SubCommand::Set => {
            // perform discovery
            println!("Set {}: {}", args.resource, args.stdin);
        }
        SubCommand::Test => {
            // perform discovery
            println!("Test {}: {}", args.resource, args.stdin);
        }
        SubCommand::FlushCache => {
            println!("FlushCache");
        }
    }
}
