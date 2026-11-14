---
description: Command line reference for the 'dsc plugin list' command
ms.date:     11/13/2026
ms.topic:    reference
title:       dsc plugin list
---

# dsc plugin list

## Synopsis

Retrieves the list of available DSC plug-ins with an optional filter.

## Syntax

```sh
dsc plugin list [Options] <PLUGIN_NAME>
```

## Description

The `list` subcommand searches for available DSC command plug-ins and returns their information.
DSC discovers plug-ins by searching the directories it searches for resource and extension
manifests, which by default are the directory that contains `dsc` and the directories in the `PATH`
or [`DSC_RESOURCE_PATH`][01] environment variable. DSC reads the files whose name ends with
`.dsc.plugin.json`, `.dsc.plugin.yaml`, or `.dsc.plugin.yml`, and the `plugins` property of manifest
lists.

DSC returns the plug-ins in the order it discovered them. That order decides which plug-in provides
a command when more than one plug-in defines the same command name: the plug-in DSC discovered first
provides the command, and the others are reported as shadowed.

DSC doesn't return a plug-in when:

- The manifest can't be loaded or isn't valid. DSC writes a warning that names the file.
- The `condition` of the manifest evaluates to `false`.
- The command name of the plug-in is the name or the alias of a built-in command, like `config`.
  DSC writes a warning, because built-in commands always take precedence and the plug-in could
  never run.
- Another manifest defines a higher version of the same plug-in type.

When the `plugins.enabled` setting is `false`, DSC doesn't discover plug-ins and the command
returns nothing.

If the command includes the `PLUGIN_NAME` argument, DSC filters the list of discovered plug-ins
before returning them. Filters are always applied after plug-in discovery.

Listing plug-ins never runs a plug-in.

## Examples

### Example 1 - List all plug-ins

Without any filters, the command returns every discovered DSC plug-in.

```sh
dsc plugin list
```

```Output
Command  Type                Version  Description
----------------------------------------------------------------------------------------------------
lint     Microsoft.DSC/Lint  3.4.0    Check configuration documents for problems without applying them
graph    Contoso.DSC/Graph   0.1.0    Show the dependency graph of a configuration document
```

### Example 2 - List plug-ins with a matching type name

When the `PLUGIN_NAME` argument includes a wildcard, the command returns every plug-in with a
matching type name.

```sh
dsc plugin list Contoso*
```

```Output
Command  Type               Version  Description
-------------------------------------------------------------------------------------
graph    Contoso.DSC/Graph  0.1.0    Show the dependency graph of a configuration document
```

### Example 3 - Find out which plug-in provides a command

When two plug-ins define the same command name, the summary table marks the plug-in that doesn't
provide the command as shadowed.

```sh
dsc plugin list
```

```Output
Command           Type                Version  Description
--------------------------------------------------------------------------------------------------
graph             Contoso.DSC/Graph   0.1.0    Show the dependency graph of a configuration document
graph (shadowed)  Tailspin.DSC/Graph  1.2.0    Render a configuration document as a diagram
```

To see where each plug-in is defined, return the data as JSON or YAML. The `shadowed` property
indicates whether the plug-in is shadowed and the `path` property is the path to its manifest.

To run a shadowed plug-in, specify its type name instead of the command name, like
`dsc Tailspin.DSC/Graph`.

```sh
dsc plugin list --output-format yaml
```

## Arguments

### PLUGIN_NAME

Specifies an optional filter to apply for the type names of discovered DSC plug-ins. The filter can
include wildcards (`*`). The filter isn't case-sensitive.

When this argument is specified, DSC filters the results to include only plug-ins where the plug-in
type name matches the filter.

For example, specifying the filter `Microsoft.*` returns only the plug-ins published by Microsoft.
Specifying the filter `*Lint*` returns any plug-in with the string `Lint` in its type name,
regardless of the casing.

The filter applies to the type name of the plug-in, not to the name of the command it provides.

The value for this argument must be either a valid [fully qualified type name][02] or a valid
wildcard type name.

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

- `json` to emit the data as a [JSON Line][03].
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

This command returns a formatted array containing an object for each plug-in that includes the
plug-in's type, version, command name, manifest, and other metadata. For more information, see
[dsc plugin list result schema][04].

If the output of the command isn't captured or redirected, it displays in the console by default as
a summary table for the returned plug-ins. The summary table includes the following columns,
displayed in the listed order:

- **Command** - The name of the command the plug-in defines, which you specify after `dsc` to run
  the plug-in. The name is followed by `(shadowed)` when a plug-in that DSC discovered earlier
  provides the same command.
- **Type** - The fully qualified type name of the plug-in.
- **Version** - The semantic version of the plug-in.
- **Description** - The short description of the command.

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: ../index.md#dsc_resource_path
[02]: ../../schemas/definitions/resourceType.md
[03]: https://jsonlines.org/
[04]: ../../schemas/outputs/plugin/list.md
