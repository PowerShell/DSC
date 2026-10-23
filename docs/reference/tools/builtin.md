---
description: >-
  Lists the tools that ship with DSC, their purpose, and links to the reference documentation for
  each tool.
ms.date:     10/23/2026
ms.topic:    reference
title:       osinfo
---

# DSC tools overview

Microsoft's Desired State Configuration (DSC) platform includes command-line tools for early
feedback and functionality.

The following table describes the tools included in current releases of DSC and the platforms those
tools are available for:

| Executable | Platforms             | Description                                                        |
|:-----------|:----------------------|:-------------------------------------------------------------------|
| `dsclint`  | Linux, macOS, Windows | Checks configuration documents for problems without applying them. |
| `osinfo`   | Linux, macOS, Windows | Returns information about the operating system as JSON.            |
| `registry` | Windows               | Manages registry keys and values.                                  |

## dsclint

The `dsclint` command checks a DSC configuration document for problems without applying it. It
reports duplicate resources, invalid expressions, unresolved dependencies, invalid parameters,
deprecated features, and, with resource discovery, resource types that aren't installed. `dsclint`
is available in the DSC release for all supported platforms.

For more information about the command, see [dsclint command reference][05].

## osinfo

The `osinfo` command returns information about the operating system as JSON. `osinfo` is
available in the DSC release for all supported platforms. DSC includes this command to provide the
`Microsoft/OSInfo` DSC resource.

For more information about the command, see [osinfo command reference][01]. For more
information about the resource, see [Microsoft/OSInfo][02].

## registry

The `registry` command manages Windows Registry keys and values. `registry` is only available in
DSC releases for Windows. DSC includes this command to provide the `Microsoft.Windows/Registry` DSC
resource.

For more information about the command, see [registry command reference][03]. For
more information about the resource, see [Microsoft.Windows/Registry][04].

<!-- Link reference definitions -->
[01]: ./osinfo.md
[02]: ../resources/Microsoft/OSInfo/index.md
[03]: ./registry/index.md
[04]: ../resources/Microsoft/Windows/Registry/index.md
[05]: ./dsclint/index.md
