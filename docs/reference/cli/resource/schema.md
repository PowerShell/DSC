---
description: Command line reference for the 'dsc resource schema' command
ms.date:     3/05/2025
ms.topic:    reference
title:       dsc resource schema
---

# dsc resource schema

## Synopsis

Returns the JSON Schema for validating instances of a resource.

## Syntax

```sh
dsc resource schema [Options] --resource <RESOURCE>
```

## Description

The `schema` subcommand returns the JSON schema for a instances of a specific DSC Resource. DSC
uses these schemas to validate the input for the `get`, `set`, and `test` subcommands and when
validating the instances in a DSC Configuration document.

Integrating tools may use these schemas for validation or to enhance the configuration authoring
experience. A resource's instance schema defines the valid structure for an instance, including
which properties are mandatory and what their values should be. The instance schemas may also
include lightweight documentation for the properties with the `title` and `description` keywords.

## Examples

### Example 1 - Get the schema for a resource

<a id="example-1"></a>

This example returns the schema for the `OSInfo` command resource.

```sh
dsc resource schema --resource Microsoft/OSInfo
```

```yaml
$schema: http://json-schema.org/draft-07/schema#
$id: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/resources/Microsoft/OSInfo/v0.1.0/schema.json
title: OsInfo
description: |
  Returns information about the operating system.

  https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource
markdownDescription: |
  The `Microsoft/OSInfo` resource enables you to assert whether a machine meets criteria related to
  the operating system. The resource is only capable of assertions. It doesn't implement the set
  operation and can't configure the operating system.

  [Online documentation][01]

  [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource
type: object
required: []
additionalProperties: false
properties:
  $id:
    type: string
    readOnly: true
    title: Data Type ID
    description: |
      Returns the unique ID for the OSInfo instance data type.

      https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#id
    markdownDescription: |
      Returns the unique ID for the OSInfo instance data type.

      [Online documentation][01]

      [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#id
  architecture:
    type: string
    title: Processor architecture
    description: |
      Defines the processor architecture as reported by 'uname -m' on the operating system.

      https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#architecture
    markdownDescription: |
      Defines the processor architecture as reported by `uname -m` on the operating system.

      [Online documentation][01]

      [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#architecture
  bitness:
    type: string
    enum:
    - '32'
    - '64'
    - unknown
    title: Operating system bitness
    description: |
      Defines whether the operating system is a 32-bit or 64-bit operating system.

      https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#bitness
    markdownDescription: |
      Defines whether the operating system is a 32-bit or 64-bit operating system.

      [Online documentation][01]

      [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#bitness
  codename:
    type: string
    title: Linux codename
    description: |
      Defines the codename for the operating system as returned from 'lsb_release --codename'.

      https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#codename
    markdownDescription: |
      Defines the codename for the operating system as returned from `lsb_release --codename`.

      [Online documentation][01]

      [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#codename
  edition:
    type: string
    title: Windows edition
    description: |
      Defines the operating system edition, like 'Windows 11' or 'Windows Server 2016'.

      https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#edition
    markdownDescription: |
      Defines the operating system edition, like `Windows 11` or `Windows Server 2016`.

      [Online documentation][01]

      [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#edition
  family:
    type: string
    enum:
    - Linux
    - macOS
    - Windows
    title: Operating system family
    description: |
      Defines whether the operating system is Linux, macOS, or Windows.

      https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#family
    markdownDescription: |
      Defines whether the operating system is Linux, macOS, or Windows.

      [Online documentation][01]

      [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#family
  version:
    type: string
    title: Operating system version
    description: |
      Defines the version of the operating system as a string.

      https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#version
    markdownDescription: |
      Defines the version of the operating system as a string.

      [Online documentation][01]

      [01]: https://learn.microsoft.com/powershell/dsc/reference/microsoft/osinfo/resource#version
```

## Options

### -r, --resource

<a id="-r"></a>
<a id="--resource"></a>

Specifies the fully qualified type name of the DSC Resource to retrieve the instance schema for,
like `Microsoft.Windows/Registry`.

The fully qualified type name syntax is: `<owner>[.<group>][.<area>]/<name>`, where:

- The `owner` is the maintaining author or organization for the resource.
- The `group` and `area` are optional name components that enable namespacing for a resource.
- The `name` identifies the component the resource manages.

```yaml
Type        : string
Mandatory   : true
LongSyntax  : --resource <RESOURCE>
ShortSyntax : -r <RESOURCE>
```

### -o, --output-format

<a id="-o"></a>
<a id="--output-format"></a>

The `--output-format` option controls which format DSC uses for the data the command returns. The
available formats are:

- `json` to emit the data as a [JSON Line][aa].
- `pretty-json` to emit the data as JSON with newlines, indentation, and spaces for readability.
- `yaml` to emit the data as YAML.

The default output format depends on whether DSC detects that the output is being redirected or
captured as a variable:

- If the command isn't being redirected or captured, DSC displays the output as the `yaml` format
  in the console.
- If the command output is redirected or captured, DSC emits the data as the `json` format to
  stdout.

When you use this option, DSC uses the specified format regardless of whether the command is being
redirected or captured.

When the command isn't redirected or captured, the output in the console is formatted for improved
readability. When the command isn't redirected or captured, the output include terminal sequences
for formatting.

```yaml
Type        : string
Mandatory   : false
ValidValues : [json, pretty-json, yaml]
LongSyntax  : --output-format <OUTPUT_FORMAT>
ShortSyntax : -o <OUTPUT_FORMAT>
```

[aa]: https://jsonlines.org/

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

This command returns formatted data representing the JSON schema for an instance of the specified
DSC Resource.

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).