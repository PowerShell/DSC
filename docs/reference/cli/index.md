---
description: Command line reference for the 'dsc' command
ms.date:     05/05/2026
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

### mcp

The `mcp` command starts DSC as a Model Context Protocol (MCP) server. For more information, see
[dsc mcp][04].

### plugin

The `plugin` command manages DSC command plug-ins. You can use it to:

- List the available plug-ins with the command each one provides.

For more information, see [dsc plugin][07]. For information about running a command that a plug-in
provides, see [Plug-in commands](#plug-in-commands).

### resource

The `resource` command manages a DSC Resource. You can use it to:

- List the available resources.
- Get the JSON schema for a resource's instances.
- Get the current state of a resource instance.
- Test whether a resource instance is in the desired state.
- Set a resource instance to the desired state.

For more information, see [dsc resource][05].

### schema

The `schema` command returns the JSON schema for a specific DSC type. For more information, see
[dsc schema][06].

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

For a command that a plug-in provides, `dsc help <COMMAND>` runs the plug-in with the `--help`
option, so the plug-in displays its own help.

## Plug-in commands

A command plug-in adds a command to `dsc` without being part of the DSC executable. When the first
argument to `dsc` isn't one of the built-in [commands](#commands), DSC looks for a plug-in that
defines a command with that name and runs it:

```sh
dsc <PLUGIN_COMMAND> [<PLUGIN_ARGUMENTS>...]
```

For example, a plug-in that defines the command `lint` is invoked as `dsc lint`. Use the
[dsc plugin list][08] command to see the available plug-ins, or `dsc --help`, which lists the
plug-in commands after the built-in commands.

A plug-in is defined by a plug-in manifest, a file whose name ends with `.dsc.plugin.json`,
`.dsc.plugin.yaml`, or `.dsc.plugin.yml`. DSC discovers plug-in manifests in the same directories
as resource and extension manifests. For more information about defining a plug-in, see the
[DSC plug-in manifest schema reference][09].

DSC runs a plug-in command with the following behavior:

- Built-in commands always take precedence. A plug-in can't replace a built-in command.
- DSC only searches for plug-ins when the command isn't built in, when it displays the top-level
  help, when it lists plug-ins, and when it generates a completion script. Built-in commands never
  search for plug-ins.
- DSC passes every argument after the command name to the plug-in as written, including `--help`.
  DSC doesn't interpret those arguments.
- The plug-in runs in the current working directory with the standard input, output, and error
  streams of `dsc`. DSC doesn't modify the output of the plug-in.
- DSC exits with the exit code of the plug-in. The meaning of an exit code is defined by the
  plug-in, not by the [exit codes](#exit-codes) of `dsc`.
- When more than one plug-in defines the same command name, the plug-in DSC discovers first
  provides the command and DSC writes a warning that names the others, which are shadowed. To run
  a specific plug-in, shadowed or not, specify its fully qualified type name instead of the command
  name, like `dsc OpenDSC/Lint`. DSC doesn't write the warning when you use the type name.
- When no plug-in defines the command, DSC reports the command as an unrecognized subcommand and
  exits with code `2`.

Options for `dsc` itself, like [`--trace-level`](#--trace-level), must be specified before the
command name. DSC makes them available to the plug-in as environment variables. DSC sets the
following environment variables for a plug-in:

| Environment variable       | Value                                                                               |
| :------------------------- | :---------------------------------------------------------------------------------- |
| `DSC_EXE`                  | The absolute path to the `dsc` executable that started the plug-in.                 |
| `DSC_VERSION`              | The semantic version of that `dsc` executable.                                      |
| `DSC_PLUGIN_COMMAND`       | The name the plug-in was invoked as: its command name or its type name.             |
| `DSC_PLUGIN_DIR`           | The directory that contains the plug-in manifest.                                   |
| `DSC_TRACE_LEVEL`          | The trace level in effect, from the `--trace-level` option or the settings.         |
| `DSC_TRACE_FORMAT`         | The trace format in effect, from the `--trace-format` option or the settings.       |
| `DSC_PROGRESS_FORMAT`      | The value of the `--progress-format` option. Only set when the option is specified. |
| `DSC_IGNORE_SETTINGS_FILE` | Set to `1` or `0` when the `--ignore-settings-file` option is specified.            |

To prevent DSC from discovering and running plug-ins, set `plugins.enabled` to `false` in a DSC
settings file. When the setting is defined in the policy settings file, users can't override it.

## Options

### -l, --trace-level

<a id="-l"></a>
<a id="--trace-level"></a>

Defines the minimum message level DSC should emit during an operation. Messages in DSC are
categorized by their level.

The following list shows the valid message levels from highest to lowest level. When this option is
set to any value in the list, DSC emits messages at that level and above.

- `error`
- `warn` (default)
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

You can also set the trace level with the [`DSC_TRACE_LEVEL`](#dsc_trace_level) environment
variable. When you specify this option and define the environment variable the option takes
precedence over the level defined by the environment variable.

```yaml
Type         : string
Mandatory    : false
DefaultValue : warn
ValidValues  : [error, warn, info, debug, trace]
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

### DSC_RESOURCE_PATH

By default, the `dsc` command searches for DSC resource manifests in the folders defined by the
`PATH` environment variable. If the `DSC_RESOURCE_PATH` environment variable is defined, `dsc`
searches the folders in `DSC_RESOURCE_PATH` instead of `PATH`.

The `DSC_RESOURCE_PATH` environment must be an environment variable that follows the same
conventions as the `PATH` environment variable for the operating system. Separate folder paths with
a semicolon (`;`) on Windows and a colon (`:`) on other platforms.

### DSC_TRACE_LEVEL

By default, the `dsc` command uses the `warning` trace level. If the `DSC_TRACE_LEVEL` environment
variable is defined, `dsc` uses the defined value as the default trace level instead. When you
specify the [`--trace-level`](#--trace-level) option it _always_ overrides the default value and
the environment variable.

When defining the `DSC_TRACE_LEVEL` environment variable, always use the uppercase format for the
trace level:

- `ERROR`
- `WARNING`
- `INFO`
- `DEBUG`
- `TRACE`

## Exit Codes

The `dsc` command uses semantic exit codes. Each exit code represents a different result for the
execution of the command.

| Exit Code | Meaning                                                                                                 |
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

<!-- Link reference definitions -->
[01]: completer/index.md
[02]: config/index.md
[03]: function/index.md
[04]: server/index.md
[05]: resource/index.md
[06]: schema/index.md
[07]: plugin/index.md
[08]: plugin/list.md
[09]: ../schemas/plugin/manifest/root.md
