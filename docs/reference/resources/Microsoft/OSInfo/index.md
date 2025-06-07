---
description: Microsoft/OSInfo DSC resource reference documentation
ms.date:     03/25/2025
ms.topic:    reference
title:       Microsoft/OSInfo
---

# Microsoft/OSInfo

## Synopsis

Returns information about the operating system.

> [!IMPORTANT]
> The `osinfo` command and `Microsoft/OSInfo` resource are a proof-of-concept example for use with
> DSC. Don't use it in production.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [os, linux, windows, macos]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft/OSInfo
    properties:
      # Instance Properties
      architecture:
      bitness:
      codename:
      edition:
      family:
      version:
```

## Description

The `Microsoft/OSInfo` resource enables you to assert whether a machine meets criteria related to
the operating system. The resource is only capable of assertions. It doesn't implement the set
operation and can't configure the operating system.

The resource doesn't implement the [test operation][01]. It relies on the synthetic testing feature
of DSC instead. The synthetic test uses a case-sensitive equivalency comparison between the actual
state of the instance properties and the desired state. If any property value isn't an exact match,
DSC considers the instance to be out of the desired state.

The instance properties returned by this resource depend on the operating system `family` as
listed in the following table:

| `family`  |                Returned instance properties                |
| :-------: | :--------------------------------------------------------- |
|  `Linux`  | `architecture`, `bitness`, `codename`, `family`, `version` |
|  `MacOS`  | `architecture`, `bitness`, `family`, `version`             |
| `Windows` | `bitness`, `edition`, `family`, `version`                  |

> [!NOTE]
> This resource is installed with DSC itself on all platforms.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

None.

## Capabilities

This resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `export` - You can use the resource to retrieve the actual state of every instance.

This resource uses the synthetic test functionality of DSC to determine whether an instance is
in the desired state.

This resource doesn't have the `set` capability. You can't use it to modify the state of a system.

For more information about resource capabilities, see
[DSC resource capabilities][02].

## Examples

1. [Validate operating system information with dsc resource][03]
1. [Validate operating system information in a configuration][04]

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> This resource doesn't have any required
  properties.
- **Key properties:** <a id="key-properties"></a> This resource doesn't have any key properties.
- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [architecture](#architecture) - Defines the processor architecture on Linux and macOS systems.
  - [bitness](#bitness) - Defines whether the operating system is 32-bit or 64-bit.
  - [codename](#codename) - Defines the codename for Linux systems.
  - [edition](#edition) - Defines the edition for Windows systems.
  - [family](#family) - Defines whether the system is Linux, macOS, or Windows.
  - [version](#version) - Defines the version of the operating system.
- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][05].

  - [$id](#id) - Returns the unique ID for the OSInfo instance data type.

### architecture

<details><summary>Expand for <code>architecture</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the processor architecture as reported by `uname -m` on the operating system. The resource
doesn't return this property for Windows machines.

### bitness

<details><summary>Expand for <code>bitness</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
ValidValues      : ['32', '64', unknown]
```

</details>

Defines whether the operating system is a 32-bit or 64-bit operating system. When the resource
can't determine this information, it returns a value of `unknown`.

### codename

<details><summary>Expand for <code>codename</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the codename for the operating system as returned from `lsb_release --codename`. The
resource only returns this property for Linux machines.

### edition

<details><summary>Expand for <code>edition</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the operating system edition, like `Windows 11` or `Windows Server 2016`. The resource only
returns this property for Windows machines.

### family

<details><summary>Expand for <code>family</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
ValidValues      : [Linux, macOS, Windows]
```

</details>

### version

<details><summary>Expand for <code>version</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the version of the operating system as a string.

### $id

<details><summary>Expand for <code>$id</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : true
IsWriteOnly      : false
ConstantValue    : https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
```

</details>

Returns the unique ID for the OSInfo instance data type.

## Exit Codes

The resource uses the following exit codes to report success and errors:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed. Review the error message for more information about the
operation failure.

## See also

- [Command line reference for the osinfo command][06]

<!-- Link references -->
[01]: ../../../../concepts/resources/overview.md#test-operations
[02]: ../../../../concepts/resources/capabilities.md
[03]: examples/validate-with-dsc-resource.md
[04]: examples/validate-in-a-configuration.md
[05]: ../../../../concepts/resources/properties.md#read-only-resource-properties
[06]: ../../../tools/osinfo.md
