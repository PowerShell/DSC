---
description: Command line reference for the 'dsc' command
ms.date:     03/25/2025
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
given shell. For more information, see [dsc completer][01].

### config

The `config` command manages a DSC Configuration document. You can use it to:

- Get the current state of the configuration.
- Test whether a configuration is in the desired state.
- Set a configuration to the desired state.

For more information, see [dsc config][02].

### function

The `function` command manages DSC functions. You can use it to:

- List the available functions with their descriptions and argument requirements.

For more information, see [dsc function][03]

### resource

The `resource` command manages a DSC Resource. You can use it to:

- List the available resources.
- Get the JSON schema for a resource's instances.
- Get the current state of a resource instance.
- Test whether a resource instance is in the desired state.
- Set a resource instance to the desired state.

For more information, see [dsc resource][04]

### schema

The `schema` command returns the JSON schema for a specific DSC type. For more information, see
[dsc schema][05].

### help

The `help` command returns help information for dsc, a command, or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
dsc help <COMMAND> [<SUBCOMMAND>]
```

For example, `dsc help config` gets the help for the `config` subcommand. `dsc help config set`
gets the help for the `config set` subcommand.

You can also use the [--help](#--help) option on a command to display the help information. For
example, `dsc config --help` or `dsc config set --help`.

## Options

### -l, --trace-level

<a id="-l"></a>
<a id="--trace-level"></a>

Defines the minimum message level DSC should emit during an operation. Messages in DSC are
categorized by their level.

The following list shows the valid message levels from highest to lowest level. When this option is
set to any value in the list, DSC emits messages at that level and above.

- `error`
- `warning` (default)
- `info`
- `debug`
- `trace`

> [!WARNING]
> The `trace` level output emits all JSON input/output that DSC processes during execution. DSC
> doesn't sanitize the JSON before emitting it. This trace level is only intended for developer
> use. Never redirect `trace` level output to storage as it might contain sensitive information.

For example, when the log level is `debug`, DSC emits messages for every log level except `trace`.
When the log level is `error`, DSC only emits error messages. DSC ignores every message with a
lower log level.

```yaml
Type         : string
Mandatory    : false
DefaultValue : warning
ValidValues  : [error, warning, info, debug, trace]
LongSyntax   : --trace-level <TRACE_LEVEL>
ShortSyntax  : -l <TRACE_LEVEL>
```

### -f, --trace-format

<a id="-f"></a>
<a id="--trace-format"></a>

Defines the output format to use when emitting trace messages on stderr. DSC supports the following
formats:

- `default` - Emits the message with ANSI console coloring for the timestamp, message level, and
  line number.
- `plaintext` - As `default` but without any console colors.
- `json` - Emits each message as a compressed JSON object with the timestamp, level, message, and
  line number as properties.

```yaml
Type         : string
Mandatory    : false
DefaultValue : default
ValidValues  : [default, plaintext, json]
LongSyntax   : --trace-format <TRACE_FORMAT>
ShortSyntax  : -f <TRACE_FORMAT>
```

### -p, --progress-format

<a id="-p"></a>
<a id="--progress-format"></a>

Defines the progress format to use when emitting progress messages on stderr. DSC supports the
following formats:

- `default` - Shows a progress bar if DSC detects that it's being called interactively. Otherwise,
  DSC doesn't show any progress.
- `none` - Doesn't show any progress.
- `json` - Emits progress as compressed JSON objects with the timestamp, level, message, and line
  number as properties.

```yaml
Type         : string
Mandatory    : false
DefaultValue : default
ValidValues  : [default, none, json]
LongSyntax   : --progress-format <PROGRESS_FORMAT>
ShortSyntax  : -p <PROGRESS_FORMAT>
```

### -V, --version

<a id="-v"></a>
<a id="--version"></a>

Displays the version of the application. When you specify this option, the application ignores all
options and arguments except for [--help](#--help), which overrides this option.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --version
ShortSyntax : -V
```

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all other options and arguments.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --help
ShortSyntax : -h
```

## Environment Variables

By default, the `dsc` command searches for DSC resource manifests in the folders defined by the
`PATH` environment variable. If the `DSC_RESOURCE_PATH` environment variable is defined, `dsc`
searches the folders in `DSC_RESOURCE_PATH` instead of `PATH`.

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

## Notes

DSC expects input strings to use UTF-8 encoding. When you pass input from stdin or the path to a
file, ensure that the input is encoded as UTF-8.

[01]: completer/index.md
[02]: config/index.md
[03]: function/index.md
[04]: resource/index.md
[05]: schema/index.md