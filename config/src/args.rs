pub struct Args {
    pub subcommand: SubCommand,
    pub resource: String,
    pub no_cache: bool, // don't use the cache and force a new discovery, low priority
}

pub enum SubCommand {
    List,
    Get,
    Set,
    Test,
    FlushCache,
}

impl Args {
    pub fn new() -> Self {
        // TODO: parse args and populate this struct
        Self {
            subcommand: SubCommand::List,
            resource: String::new(),
            no_cache: false,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Args::new()
    }
}
