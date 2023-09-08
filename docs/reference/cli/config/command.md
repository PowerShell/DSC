---
description: Command line reference for the 'dsc config' command
ms.date:     09/06/2023
ms.topic:    reference
title:       dsc config
---

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

### export

The `export` command generates a configuration document that defines the existing instances of a
set of resources. For more information, see [dsc config export][02].

### get

The `get` command retrieves the current state of the resource instances in a configuration
document. For more information, see [dsc config get][03].

### set

The `set` command enforces the desired state of the resource instances in a configuration document.
For more information, see [dsc config set][04].

### test

The `test` command verifies whether the resource instances in a configuration document are in the
desired state. For more information, see [dsc config test][05].

### help

The `help` command returns help information for this command or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
dsc config help [<SUBCOMMAND>]
```

For example, `dsc config help` gets the help for this command. `dsc config help set` gets the help
for the `set` subcommand.

You can also use the [--help](#-h---help) option on the command or subcommand to display the help
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
[02]: export.md
[03]: get.md
[04]: set.md
[05]: test.md
