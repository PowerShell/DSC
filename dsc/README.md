# `dsc` command for using DSC resources

## DESCRIPTION

The `dsc` command is used to discover, invoke DSC resources, and apply configuration.

## Usage

Apply configuration or invoke specific DSC resources

Usage: dsc [OPTIONS] <COMMAND>

Commands:
  config    Apply a configuration document
  resource  Invoke a specific DSC resource
  schema    Get the JSON schema for a DSC type
  help      Print this message or the help of the given subcommand(s)

Options:
  -n, --no-cache         Whether to use the cache or not
  -f, --format <FORMAT>  [possible values: json, pretty-json, yaml]
  -h, --help             Print help
  -V, --version          Print version
