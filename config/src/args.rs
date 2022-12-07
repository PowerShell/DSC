use std::io;
use std::process::exit;
use super::*;

pub struct Args {
    pub subcommand: SubCommand,
    pub resource: String,
    pub filter: String,
    pub stdin: String,
    pub no_cache: bool, // don't use the cache and force a new discovery, low priority
}

#[derive(Debug, PartialEq, Eq)]
pub enum SubCommand {
    List,
    Get,
    Set,
    Test,
    FlushCache,
}

impl Args {
    pub fn new(args: &mut dyn Iterator<Item = String>, input: &mut dyn io::Read, atty: bool) -> Self {
        let mut command_args: Vec<String> = args.skip(1).collect(); // skip the first arg which is the program name
        if command_args.is_empty() {
            eprintln!("No subcommand provided");
            show_help_and_exit();
        }

        let subcommand = match command_args[0].as_str() {
            "list" => SubCommand::List,
            "get" => SubCommand::Get,
            "set" => SubCommand::Set,
            "test" => SubCommand::Test,
            "flushcache" => SubCommand::FlushCache,
            _ => {
                eprintln!("Invalid subcommand provided");
                show_help_and_exit();
                SubCommand::List
            }
        };

        command_args.remove(0);   // remove the subcommand

        let mut no_cache = false;
        let mut resource = String::new();
        let mut filter = String::new();
        let mut stdin = Vec::new();

        if !atty {
            // only read if input is piped in and not a tty (terminal)
            input.read_to_end(&mut stdin).unwrap();
        }

        let stdin = match String::from_utf8(stdin) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Invalid UTF-8 sequence: {}", e);
                exit(EXIT_INVALID_INPUT);
            }
        };

        // go through reset of provided args 
        for arg in command_args {
            match arg.as_str() {
                "-h" | "--help" => show_help_and_exit(),
                "-n" | "--nocache" => no_cache = true,
                _ => {
                    if subcommand == SubCommand::FlushCache {
                        eprintln!("No arguments allowed for `flushcache`");
                        show_help_and_exit();
                    }

                    match subcommand {
                        SubCommand::List => {
                            if !filter.is_empty() {
                                eprintln!("Only one filter allowed");
                                show_help_and_exit();
                            }
                            filter = arg;
                        }
                        _ => {
                            if !resource.is_empty() {
                                eprintln!("Only one resource allowed");
                                show_help_and_exit();
                            }
                            resource = arg;
                        }
                    }
                }
            }
        }

        match subcommand {
            SubCommand::Set | SubCommand::Test => {
                if stdin.is_empty() {
                    eprintln!("Desired state input via stdin is required for `set` and `test`");
                    show_help_and_exit();
                }
            }
            _ => {}
        }

        Self {
            subcommand,
            resource,
            filter,
            stdin,
            no_cache,
        }
    }
}

fn show_help_and_exit() {
    eprintln!();
    eprintln!("Usage: config [subcommand] [options]");
    eprintln!("Subcommands:");
    eprintln!("  list   [filter]    - list all resources, optional filter");
    eprintln!("  get    <resource>  - invoke `get` on a resource");
    eprintln!("  set    <resource>  - invoke `set` on a resource");
    eprintln!("  test   <resource>  - invoke `test` on a resource");
    eprintln!("  flushcache         - flush the resource cache");
    eprintln!("Options:");
    eprintln!("  -h, --help");
    eprintln!("  -n, --nocache      - don't use the cache and force a new discovery");
    exit(EXIT_INVALID_ARGS);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Stdin {
        input: String,
    }

    impl Stdin {
        fn new(input: &str) -> Self {
            Self {
                input: input.to_string(),
            }
        }
    }

    impl io::Read for Stdin {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let bytes = self.input.as_bytes();
            let len = bytes.len();
            buf.clone_from_slice(bytes);
            Ok(len)
        }

        fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
            let bytes = self.input.as_bytes();
            let len = bytes.len();
            buf.extend_from_slice(bytes);
            Ok(len)
        }
    }

    #[test]
    fn test_args_list() {
        let mut args = ["config", "list", "myfilter"].iter().map(|s| s.to_string());
        let args = Args::new(&mut args, &mut Stdin::new(""), false);
        assert_eq!(args.subcommand, SubCommand::List);
        assert_eq!(args.resource, "");
        assert_eq!(args.filter, "myfilter");
        assert_eq!(args.stdin, "");
        assert_eq!(args.no_cache, false);
    }

    #[test]
    fn test_args_get() {
        let mut args = ["config", "get", "myresource"].iter().map(|s| s.to_string());
        let args = Args::new(&mut args, &mut Stdin::new("abc"), false);
        assert_eq!(args.subcommand, SubCommand::Get);
        assert_eq!(args.resource, "myresource");
        assert_eq!(args.filter, "");
        assert_eq!(args.stdin, "abc");
        assert_eq!(args.no_cache, false);
    }

    #[test]
    fn test_args_set() {
        let mut args = ["config", "set", "myresource"].iter().map(|s| s.to_string());
        let args = Args::new(&mut args, &mut Stdin::new("abc"), false);
        assert_eq!(args.subcommand, SubCommand::Set);
        assert_eq!(args.resource, "myresource");
        assert_eq!(args.filter, "");
        assert_eq!(args.stdin, "abc");
        assert_eq!(args.no_cache, false);
    }

    #[test]
    fn test_args_test() {
        let mut args = ["config", "test", "myresource"].iter().map(|s| s.to_string());
        let args = Args::new(&mut args, &mut Stdin::new("abc"), false);
        assert_eq!(args.subcommand, SubCommand::Test);
        assert_eq!(args.resource, "myresource");
        assert_eq!(args.filter, "");
        assert_eq!(args.stdin, "abc");
        assert_eq!(args.no_cache, false);
    }

    #[test]
    fn test_args_flushcache() {
        let mut args = ["config", "flushcache"].iter().map(|s| s.to_string());
        let args = Args::new(&mut args, &mut Stdin::new(""), false);
        assert_eq!(args.subcommand, SubCommand::FlushCache);
        assert_eq!(args.resource, "");
        assert_eq!(args.filter, "");
        assert_eq!(args.stdin, "");
        assert_eq!(args.no_cache, false);
    }
}
