---
description: Command line reference for the 'dsc function' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc function
---

# dsc function

## Synopsis

Operations on DSC functions.

## Syntax

```sh
dsc function [Options] <COMMAND>
```

## Description

The `dsc function` command contains a subcommand for listing DSC functions.

## Commands

### list

The `list` command returns the list of available DSC functions with an optional filter. For more
information, see [dsc function list][01].

### help

The `help` command returns help information for this command or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
dsc function help [<SUBCOMMAND>]
```

For example, `dsc function help` gets the help for this command. `dsc function help list`
gets the help for the `list` subcommand.

You can also use the [--help](#--help) option on the command or subcommand to display the help
information. For example, `dsc function --help` or `dsc function list --help`.

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

[01]: ./list.md
