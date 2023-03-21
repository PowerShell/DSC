# `config` command for using DSC resources

## DESCRIPTION

The `config` command is used to discover and invoke DSC resources.

## Usage

Usage: config [subcommand] [options]
Subcommands:
  list   [filter]    - list all resources, optional filter
  get    <resource>  - invoke `get` on a resource
  set    <resource>  - invoke `set` on a resource
  test   <resource>  - invoke `test` on a resource
  flushcache         - flush the resource cache
Options:
  -h, --help
  -n, --nocache      - don't use the cache and force a new discovery
