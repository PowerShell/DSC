---
description: Command line reference for the 'dsc' command
ms.date:     10/05/2023
ms.topic:    reference
title:       dsc
---

# dsc

## Synopsis

Apply configuration or invoke specific resources to manage software components.

## Syntax

```sh
dsc [Options] <COMMAND>
```

## Commands

### completer

The `completer` command returns a shell script that, when executed, registers completions for the
given shell. For more information, see [completer][01].

### config

The `config` command manages a DSC Configuration document. You can use it to:

- Get the current state of the configuration.
- Test whether a configuration is in the desired state.
- Set a configuration to the desired state.

For more information, see [config][02].

### resource

The `resource` command manages a DSC Resource. You can use it to:

- List the available resources.
- Get the JSON schema for a resource's instances.
- Get the current state of a resource instance.
- Test whether a resource instance is in the desired state.
- Set a resource instance to the desired state.

For more information, see [resource][03]

### schema

The `schema` command returns the JSON schema for a specific DSC type. For more information, see
[schema][04].

### help

The `help` command returns help information for dsc, a command, or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
dsc help <COMMAND> [<SUBCOMMAND>]
```

For example, `dsc help config` gets the help for the `config` subcommand. `dsc help config set`
gets the help for the `config set` subcommand.

You can also use the [--help](#-h---help) option on a command to display the help information. For
example, `dsc config --help` or `dsc config set --help`.

## Options

### -f, --format

The `--format` option controls the console output format for the command. If the command output is
redirected or captured as a variable, the output is always JSON.

To set the output format for a command or subcommand, specify this option before the command, like
`dsc --format pretty-json resource list`.

```yaml
Type:         String
Mandatory:    false
DefaultValue: yaml
ValidValues:  [json, pretty-json, yaml]
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

### -i, --input

Defines input for the command as a string instead of piping input from stdin. This option is
mutually exclusive with the `--input-file` option. When you use this option, DSC ignores any input
from stdin.

To pass input for a command or subcommand, specify this option before the command, like
`dsc --input $desired resource test`.

```yaml
Type:      String
Mandatory: false
```

### -l, --logging-level

Defines the minimum log level DSC should emit during an operation. Messages in DSC are categorized
by their log level.

The following list shows the valid log levels from highest to lowest level. When this option is
set to any value in the list, DSC emits messages at that level and above.

- `Error`
- `Warning`
- `Info` (default)
- `Debug`
- `Trace`

For example, when the log level is `Debug`, DSC emits messages for every log level except `Trace`.
Dsc emits only error messages when the log level is `Error`. DSC ignores every message with a lower
log level.

```yaml
Type: String
Mandatory: false
DefaultValue: Info
ValidValues:  [Error, Warning, Info, Debug, Trace]
```

### -p, --input-file

Defines the path to a text file to read as input for the command instead of piping input from
stdin. This option is mutually exclusive with the `--input` option. When you use this option, DSC
ignores any input from stdin.

To pass a file to read as input for a command or subcommand, specify this option before the
command, like `dsc --input-file web.dsc.config.yaml config set`.

If the specified file doesn't exist, DSC raises an error.

```yaml
Type:      String
Mandatory: false
```

### -V, --version

Displays the version of the application. When you specify this option, the application ignores all
options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Environment Variables

By default, the `dsc` command searches for command-based DSC Resource manifests in the folders
defined by the `PATH` environment variable. If the `DSC_RESOURCE_PATH` environment variable is
defined, `dsc` searches the folders in `DSC_RESOURCE_PATH` instead of `PATH`.

The `DSC_RESOURCE_PATH` environment must be an environment variable that follows the same
conventions as the `PATH` environment variable for the operating system. Separate folder paths with
a semicolon (`;`) on Windows and a colon (`:`) on other platforms.

## Exit Codes

The `dsc` command uses semantic exit codes. Each exit code represents a different result for the
execution of the command.

| Exit Code |                                                 Meaning                                                 |
| :-------: | :------------------------------------------------------------------------------------------------------ |
|    `0`    | The command executed successfully without any errors.                                                   |
|    `1`    | The command failed because it received invalid arguments.                                               |
|    `2`    | The command failed because a resource raised an error.                                                  |
|    `3`    | The command failed because a value couldn't be serialized to or deserialized from JSON.                 |
|    `4`    | The command failed because input for the command wasn't valid YAML or JSON.                             |
|    `5`    | The command failed because a resource definition or instance value was invalid against its JSON schema. |
|    `6`    | The command was cancelled by a <kbd>Ctrl</kbd>+<kbd>C</kbd> interruption.                               |

[01]: completer/command.md
[02]: config/command.md
[03]: resource/command.md
[04]: schema/command.md
