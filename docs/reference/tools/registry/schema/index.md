---
description: Command line reference for the 'registry schema' command
ms.date:     03/18/2025
ms.topic:    reference
title:       registry schema
---

# registry schema

## Synopsis

Returns the JSON schema for `Microsoft.Windows/Registry` resource instances.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry schema [Options]
```

## Description

The `schema` command returns the JSON schema for instances of the `Microsoft.Windows/Registry` DSC
Resource. The `registry` command, DSC, and integrating tools use this schema to validate the
properties for resource instances.

By default, the command outputs the schema as a single line string of JSON.

For more information about defining an instance of the `Microsoft.Windows/Registry` resource, see
[Microsoft.Windows/Registry][01].

## Examples

### Example 1 - Get the compressed schema

<a id="example-1"></a>

The output is a single line containing the JSON schema without any unquoted whitespace.

```powershell
registry schema
```

### Example 2 - Get the pretty-print schema

<a id="example-2"></a>

With the `--pretty` flag, the output includes whitespace between key-value pairs, newlines after
each key-value pair and array item, and indentation to make the schema easier to read.

```powershell
registry schema --pretty
```

## Options

### -p, --pretty

<a id="-p"></a>
<a id="--pretty"></a>

Returns the schema with indentation and newlines between key-value pairs and array items. By
default, the command returns the schema compressed on a single line without any unquoted
whitespace.

```yaml
Type:      boolean
Mandatory: false
```

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      boolean
Mandatory: false
```

<!-- Link references -->
[01]: ../../../resources/microsoft/windows/registry/index.md
