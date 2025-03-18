---
description: Command line reference for the 'registry' command
ms.date:     03/25/2025
ms.topic:    reference
title:       registry
---

# registry

## Synopsis

Manage state of Windows registry

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry [Options] <COMMAND>
```

## Description

DSC includes an example command, `registry`, for managing keys and values in the Windows registry.
The command also defines a DSC resource manifest, making it available as a [DSC resource][01] on
Windows machines.

You can use `registry` to:

- Query the registry for keys and values
- Create, update, and delete registry keys and values
- Invoke the `Microsoft.Windows/Registry` resource with DSC to manage registry keys and values
  idempotently.
- Define instances of the `Microsoft.Windows/Registry` resource in DSC Configuration Documents.

For more information about using `registry` as a resource, see [Microsoft.Windows/Registry][02].

## Commands

### query

The `query` command isn't implemented yet. It returns a string that echoes the specified options.
For more information, see [query][03].

### set

The `set` command isn't implemented yet. It returns a string that echoes the specified options. For
more information, see [set][04].

### remove

The `remove` command isn't implemented yet. It returns a string that echoes the specified options.
For more information, see [remove][05].

### find

The `find` command isn't implemented yet. It returns a string that echoes the specified options.
For more information, see [find][06].

### config

The `config` command manages registry keys and values as instances of a [DSC Resource][01]. You can
use it to:

- Get the current state of a registry key or value
- Test whether a registry key or value is in the desired state
- Set a registry key or value to the desired state.

For more information, see [config][07].

### schema

The `schema` command returns the JSON schema for an instance of the `Microsoft.Windows/Registry`
DSC Resource. For more information, see [schema][08].

### help

The `help` command returns help information for `registry`, a command, or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
registry help <COMMAND> [<SUBCOMMAND>]
```

For example, `registry help config` gets the help for the `config` subcommand.
`registry help config set` gets the help for the `config set` subcommand.

You can also use the [--help](#-h---help) option on a command to display the help information. For
example, `registry config --help` or `registry config set --help`.

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

<!-- Link reference definitions -->
[01]: ../../../concepts/resources/overview.md
[02]: ../../resources/Microsoft/Windows/Registry/index.md
[03]: ./query/index.md
[04]: ./set/index.md
[05]: ./remove/index.md
[06]: ./find/index.md
[07]: ./config/index.md
[08]: ./schema/index.md
