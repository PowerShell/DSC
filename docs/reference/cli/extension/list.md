---
description: Command line reference for the 'dsc extension list' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc extension list
---

# dsc extension list

## Synopsis

Retrieves the list of available DSC extensions with an optional filter.

## Syntax

```sh
dsc extension list [Options] <EXTENSION_NAME>
```

## Description

The `list` subcommand searches for available DSC extensions and returns their information. DSC
discovers extensions by first searching the `PATH` or `DSC_RESOURCE_PATH` environment variable for
`.dsc.extension.json`, `.dsc.extension.yml`, and `dsc.extension.yaml` files. For more information
about the environment variables DSC uses, see [Environment variables][01]

DSC returns the list of discovered extensions with their implementation information and metadata. If
the command includes the `EXTENSION_NAME` argument, DSC filters the list of discovered extensions
before returning them. Filters are always applied after extension discovery.

## Examples

### Example 1 - List all extensions

Without any filters, the command returns every discovered DSC extension.

```sh
dsc extension list
```

```Output
Type                             Version  Capabilities  Description
----------------------------------------------------------------------------------------------------------
Microsoft.Windows.Appx/Discover  0.1.0    d             Discovers DSC resources packaged as Appx packages.
```

### Example 2 - List a specific extension

When the `EXTENSION_NAME` argument doesn't include a wildcard, the command returns only the extension
with the specified type name.

```sh
dsc extension list Microsoft.Windows.Appx/Discover
```

```Output
Type                             Version  Capabilities  Description
----------------------------------------------------------------------------------------------------------
Microsoft.Windows.Appx/Discover  0.1.0    d             Discovers DSC resources packaged as Appx packages.
```

### Example 3 - List extensions with a matching type name

When the `EXTENSION_NAME` argument includes a wildcard, the command returns every extension with a
matching type name.

```sh
dsc extension list Microsoft*
```

```Output
Type                             Version  Capabilities  Description
----------------------------------------------------------------------------------------------------------
Microsoft.Windows.Appx/Discover  0.1.0    d             Discovers DSC resources packaged as Appx packages.
```

## Arguments

### EXTENSION_NAME

Specifies an optional filter to apply for the type names of discovered DSC extensions. The filter
can include wildcards (`*`). The filter isn't case-sensitive.

When this argument is specified, DSC filters the results to include only extensions where the
extension type name matches the filter.

For example, specifying the filter `Microsoft.*` returns only the extensions published by
Microsoft. Specifying the filter `*Windows*` returns any extension with the string `Windows` in its
name, regardless of the casing.

```yaml
Type      : string
Mandatory : false
```

## Options

### -o, --output-format

<a id="-o"></a>
<a id="--output-format"></a>

The `--output-format` option controls which format DSC uses for the data the command returns. The
available formats are:

- `json` to emit the data as a [JSON Line][02].
- `pretty-json` to emit the data as JSON with newlines, indentation, and spaces for readability.
- `yaml` to emit the data as YAML.
- `table-no-truncate` to emit the data as a summary table without truncating each line to the
  current console width.

The default output format depends on whether DSC detects that the output is being redirected or
captured as a variable:

- If the command isn't being redirected or captured, DSC displays the output as a summary table
  described in the [Output](#output) section of this document.
- If the command output is redirected or captured, DSC emits the data as the `json` format to
  stdout.

When you use this option, DSC uses the specified format regardless of whether the command is being
redirected or captured.

When the command isn't redirected or captured, the output in the console is formatted for improved
readability. When the command isn't redirected or captured, the output includes terminal sequences
for formatting.

```yaml
Type        : string
Mandatory   : false
ValidValues : [json, pretty-json, yaml, table-no-truncate]
LongSyntax  : --output-format <OUTPUT_FORMAT>
ShortSyntax : -o <OUTPUT_FORMAT>
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

## Output

This command returns a formatted array containing an object for each extension that includes the
extension's type, version, manifest settings, and other metadata. For more information, see
[dsc extension list result schema][03].

If the output of the command isn't captured or redirected, it displays in the console by default as
a summary table for the returned extensions. The summary table includes the following columns,
displayed in the listed order:

- **Type** - The fully qualified type name of the extension.
- **Version** - The semantic version of the extension.
- **Capabilities** - A display of the extension's [capabilities][04] as flags. The capabilities are
  displayed in the following order, using a `-` instead of the appropriate letter if the extension
  doesn't have a specific capability:

  - `d` indicates that the extension has the [discover capability][05].

  For example, the `icrosoft.Windows.Appx/Discover` extension has the following capabilities: `d`,
  indicating it has the `discover` capability.
- **Description** - The short description of the extension's purpose and usage.

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: ../index.md#environment-variables
[02]: https://jsonlines.org/
[03]: ../../schemas/outputs/extension/list.md
[04]: ../../schemas/outputs/extension/list.md#capabilities
[05]: ../../schemas/outputs/extension/list.md#capability-discover
