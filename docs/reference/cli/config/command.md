# dsc config

## Synopsis

Apply a configuration document.

## Syntax

```sh
dsc config [Options] <COMMAND>
```

## Description

The `dsc config` command includes subcommands for managing the resource instances defined in a DSC
configuration document. To manage resources directly, see the [dsc resource][01] command.

## Commands

### get

The `get` command retrieves the current state of the resource instances in a configuration
document. For more information, see [dsc config get][02].

### set

The `set` command enforces the desired state of the resource instances in a configuration document.
For more information, see [dsc config set][03].

### test

The `test` command verifies whether the resource instances in a configuration document are in the
desired state. For more information, see [dsc config test][04].

### help

The `help` command returns help information for this command or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
dsc config help [<SUBCOMMAND>]
```

For example, `dsc config help` gets the help for this command. `dsc config help set` gets the help
for the `set` subcommand.

You can also use the [--help](#h---help) option on the command or subcommand to display the help
information. For example, `dsc config --help` or `dsc config set --help`.

## Options

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

[01]: ../resource/command.md
[02]: get.md
[03]: set.md
[04]: test.md
