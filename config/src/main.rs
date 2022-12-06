pub mod args;
pub mod discovery;
pub mod dscresources;
pub mod dscerror;

use args::*;

fn main() {
    let args = Args::new();
    match args.subcommand {
        SubCommand::List => {
            // perform discovery
            println!("List");
        }
        SubCommand::Get => {
            // perform discovery
            println!("Get");
        }
        SubCommand::Set => {
            // perform discovery
            println!("Set");
        }
        SubCommand::Test => {
            // perform discovery
            println!("Test");
        }
        SubCommand::FlushCache => {
            println!("FlushCache");
        }
    }
}
