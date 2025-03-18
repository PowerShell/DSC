---
description: >-
  Lists the tools that ship with DSC, their purpose, and links to the reference documentation for
  each tool.
ms.date:     03/18/2025
ms.topic:    reference
title:       osinfo
---

# DSC tools overview

Microsoft's Desired State Configuration (DSC) platform includes commandline tools for early
feedback and functionality.

The following table describes the tools included in current releases of DSC and the platforms those
tools are available for:

| Executable | Platforms             | Description                                             |
|:-----------|:----------------------|:--------------------------------------------------------|
| `osinfo`   | Linux, macOS, Windows | Returns information about the operating system as JSON. |
| `registry` | Windows               | Manages registry keys and values.                       |

## osinfo

The `osinfo` command returns information about the operating system as JSON. `osinfo` is
available in the DSC release for all supported platforms. DSC includes this command to provide the
`Microsoft/OSInfo` DSC resource.

For more information about the command, see [osinfo command reference](./osinfo.md). For more
information about the resource, see [Microsoft/OSInfo](../resources/microsoft/osinfo/index.md).

## registry

The `registry` command manages Windows Registry keys and values. `registry` is only available in
DSC releases for Windows. DSC includes this command to provide the `Microsoft.Windows/Registry` DSC
resource.

For more information about the command, see [registry command reference](./registry/index.md). For
more information about the resource, see [Microsoft.Windows/Registry](../resources/microsoft/windows/registry/index.md).

<!-- Link reference definitions -->
