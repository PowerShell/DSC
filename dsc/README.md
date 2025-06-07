# `dsc` command for using DSC resources

## DESCRIPTION

The `dsc` command is used to discover, invoke DSC resources, and apply configuration.

## Usage

```plaintext
Apply configuration or invoke specific DSC resources

Usage: dsc.exe [OPTIONS] <COMMAND>

Commands:
  completer  Generate a shell completion script
  config     Apply a configuration document
  extension  Operations on DSC extensions
  resource   Invoke a specific DSC resource
  schema     Get the JSON schema for a DSC type
  help       Print this message or the help of the given subcommand(s)

Options:
  -l, --trace-level <TRACE_LEVEL>
          Trace level to use [possible values: error, warn, info, debug, trace]
  -t, --trace-format <TRACE_FORMAT>
          Trace format to use [possible values: default, plaintext, json]
  -p, --progress-format <PROGRESS_FORMAT>
          Progress format to use [possible values: default, none, json]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```
