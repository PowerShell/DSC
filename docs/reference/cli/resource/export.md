---
description: Command line reference for the 'dsc resource export' command
ms.date:     09/06/2023
ms.topic:    reference
title:       dsc resource export
---

# dsc resource export

## Synopsis

Generates a configuration document that defines the existing instances of a resource.

## Syntax

```sh
dsc resource export [Options] --resource <RESOURCE>
```

## Description

The `export` subcommand generates a configuration document that includes every instance of a
specific resource. The resource must be specified with the `--resource` option.

Only specify exportable resources with a resource manifest that defines the [export][01] section in
the input configuration. If the specified resource type isn't exportable, DSC raises an error.

## Options

### -r, --resource

Specifies the fully qualified type name of the DSC Resource to export, like
`Microsoft.Windows/Registry`.

The fully qualified type name syntax is: `<owner>[.<group>][.<area>]/<name>`, where:

- The `owner` is the maintaining author or organization for the resource.
- The `group` and `area` are optional name components that enable namespacing for a resource.
- The `name` identifies the component the resource manages.

```yaml
Type:      String
Mandatory: true
```

### -f, --format

The `--format` option controls the console output format for the command. If the command output is
redirected or captured as a variable, the output is always JSON.

```yaml
Type:         String
Mandatory:    false
DefaultValue: yaml
ValidValues:  [json, pretty-json, yaml]
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

This command returns JSON output that defines a configuration document including every instance of
the resources declared in the input configuration. For more information, see
[DSC Configuration document schema reference][02].

[01]: ../../schemas/resource/manifest/export.md
[02]: ../../schemas/config/document.md
