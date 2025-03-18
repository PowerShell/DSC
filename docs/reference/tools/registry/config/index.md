---
description: Command line reference for the 'registry config' command
ms.date:     03/25/2025
ms.topic:    reference
title:       registry config
---

# registry config

## Synopsis

Manage registry configuration.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry config [Options] <COMMAND>
```

## Description

The `registry config` commands manage registry keys and values as instances of the
`Microsoft.Windows/Registry` DSC Resource. They expect to receive an instance as JSON over stdin
and are usable for invoking the get, test, and set operations for the resource.

For more information about using `registry` with DSC, see [Microsoft.Windows/Registry][01].

## Commands

### get

The `get` command returns the current state of a registry key or value. For more information, see
[get][02].

### set

The `set` command enforces the desired state for a registry key or value. For more information, see
[set][03].

### delete

The `delete` command validates wether a registry key or value is in the desired state. For more
information, see [delete][04].

### help

The `help` command returns help information for this command or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
registry config help [<SUBCOMMAND>]
```

For example, `registry config help` gets the help for this command. `registry config help get`
gets the help for the `get` subcommand.

You can also use the [--help](#-h---help) option on the command or subcommand to display the help
information. For example, `registry config --help` or `registry config set --help`.

## Options

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

### -V, --version

<a id="-v"></a>
<a id="--version"></a>

Displays the version of the application. When you specify this option, the application ignores all
options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

<!-- Link references -->
[01]: ../../../resources/Microsoft/Windows/Registry/index.md
[02]: ./get.md
[03]: ./set.md
[04]: ./delete.md
