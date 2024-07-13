---
description: Command line reference for the 'dsc resource schema' command
ms.date:     06/24/2024
ms.topic:    reference
title:       dsc resource schema
---

# dsc resource schema

## Synopsis

Returns the JSON Schema for instances of a resource.

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

This example returns the schema for the `OSInfo` command-based DSC Resource.

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

Specifies the fully qualified type name of the DSC Resource to retrieve the instance schema from,
like `Microsoft.Windows/Registry`.

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

This command returns a JSON object representing the JSON schema for an instance of the specified
DSC Resource.
