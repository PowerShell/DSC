use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "sshdconfig", version = "0.1.0", about = "Manage state of sshd_config", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands
}

#[derive(Subcommand)]
pub enum Commands {
    #[clap(name = "get", about = "get sshd_config settings", arg_required_else_help = false)]
    Get {
        #[clap(long = "file", short = 'f', help = "filepath for sshd_config options")]
        input_config_path: Option<String>,
        #[clap(long = "json", short = 'j', help = "json for sshd_config options")]
        input_config_json: Option<String>,
        #[clap(long = "path", short = 'p', help = "sshd_config filepath to retrieve options from")]
        curr_config_path: Option<String>
    },

    #[clap(name = "set", about = "set sshd_config settings", arg_required_else_help = true)]
    Set {
        #[clap(long = "file", short = 'f', help = "new sshd_config filepath")]
        input_config_path: Option<String>,
        #[clap(long = "json", short = 'j', help = "new sshd_config json")]
        input_config_json: Option<String>,
        #[clap(long = "path", short = 'p', help = "existing sshd_config filepath to apply changes to")]
        curr_config_path: Option<String>
    },

    #[clap(name = "test", about = "compare existing sshd_config settings with input settings", arg_required_else_help = true)]
    Test {
        #[clap(long = "file", short = 'f', help = "new sshd_config filepath")]
        input_config_path: Option<String>,
        #[clap(long = "json", short = 'j', help = "new sshd_config json")]
        input_config_json: Option<String>,
        #[clap(long = "path", short = 'p', help = "existing sshd_config filepath to compare with")]
        curr_config_path: Option<String>
    },

}