---
description: Command line reference for the 'osinfo' command
ms.date:     03/25/2025
ms.topic:    reference
title:       osinfo command reference
---

# osinfo command reference

## Synopsis

Returns information about the operating system.

> [!IMPORTANT]
> The `osinfo` command and `Microsoft/OSInfo` resource are a proof-of-concept example for use with
> DSC. Don't use it in production.

## Syntax

```sh
osinfo
```

## Description

The `osinfo` command returns information about the operating system as a single line of compressed
JSON without any whitespace. The command doesn't accept any options or arguments.

The properties of the output JSON object are the properties for the `Microsoft/OSInfo` DSC
Resource. For more information about those properties and using the resource, see
[Microsoft/OSInfo][01].

## Examples

### Example 1 - Get operating system information

Call the command to return information about the operating system.

```sh
osinfo
```

### [Linux](#tab/linux)

```Output
{"$id":"https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json","family":"Linux","version":"20.04","codename":"focal","bitness":"64","architecture":"x86_64"}
```

The following code block shows the output with newlines and indentation for readability.

```json
{
    "$id":          "https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json",
    "family":       "Linux",
    "version":      "20.04",
    "codename":     "focal",
    "bitness":      "64",
    "architecture": "x86_64"
}
```

### [macOS](#tab/macos)

```Output
{"$id":"https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json","family":"MacOS","version":"13.5.0","bitness":"64","architecture":"arm64"}
```

The following code block shows the output with newlines and indentation for readability.

```json
{
    "$id":          "https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json",
    "family":       "MacOS",
    "version":      "13.5.0",
    "bitness":      "64",
    "architecture": "arm64"
}
```

### [Windows](#tab/windows)

```Output
{"$id":"https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json","family":"Windows","version":"10.0.22621","edition":"Windows 11 Enterprise","bitness":"64"}
```

The following code block shows the output with newlines and indentation for readability.

```json
{
    "$id":     "https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json",
    "family":  "Windows",
    "version": "10.0.22621",
    "edition": "Windows 11 Enterprise",
    "bitness": "64"
}
```

<!-- Link references -->
[01]: ../resources/Microsoft/OSInfo/index.md
