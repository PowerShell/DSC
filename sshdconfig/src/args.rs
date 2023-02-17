use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "sshdconfig", version = "0.1.0", about = "sshd_config management tool", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(about = "Get sshd_config", arg_required_else_help = false)]
    Get {
        #[clap(long = "file", short = 'f')]
        input_config_path: Option<String>,
        #[clap(long = "json", short = 'j')]
        input_config_json: Option<String>,
        #[clap(long = "path", short = 'p')]
        curr_config_path: Option<String>
    },

    #[clap(about = "Set sshd_config", arg_required_else_help = false)]
    Set {
        #[clap(long = "file", short = 'f')]
        input_config_path: Option<String>,
        #[clap(long = "json", short = 'j')]
        input_config_json: Option<String>,
        #[clap(long = "path", short = 'p')]
        curr_config_path: Option<String>
    },

    #[clap(about = "Test sshd_config", arg_required_else_help = false)]
    Test {
        #[clap(long = "file", short = 'f')]
        input_config_path: Option<String>,
        #[clap(long = "json", short = 'j')]
        input_config_json: Option<String>,
        #[clap(long = "path", short = 'p')]
        curr_config_path: Option<String>
    },

}