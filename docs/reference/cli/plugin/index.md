---
description: Command line reference for the 'dsc plugin' command
ms.date:     11/13/2026
ms.topic:    reference
title:       dsc plugin
---

# dsc plugin

## Synopsis

Operations on DSC plug-ins.

## Syntax

```sh
dsc plugin [Options] <COMMAND>
```

## Description

The `dsc plugin` command contains a subcommand for listing DSC command plug-ins.

A command plug-in adds a top-level command to DSC without being part of the DSC executable. For
example, a plug-in that defines the command `lint` is invoked as `dsc lint`. For more information
about how DSC discovers and runs plug-ins, see [Plug-in commands][01].

## Commands

### list

The `list` command returns the list of available DSC plug-ins with an optional filter. For more
information, see [dsc plugin list][02].

### help

The `help` command returns help information for this command or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
dsc plugin help [<SUBCOMMAND>]
```

For example, `dsc plugin help` gets the help for this command. `dsc plugin help list` gets the help
for the `list` subcommand.

You can also use the [--help](#--help) option on the command or subcommand to display the help
information. For example, `dsc plugin --help` or `dsc plugin list --help`.

To get the help for a command that a plug-in provides, use `dsc help <COMMAND>` or
`dsc <COMMAND> --help`. The plug-in prints its own help.

## Options

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

[01]: ../index.md#plug-in-commands
[02]: ./list.md
