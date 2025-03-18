---
description: Command line reference for the 'registry config get' command
ms.date:     03/18/2025
ms.topic:    reference
title:       registry config get
---

# registry config get

## Synopsis

Retrieve registry configuration.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry config get  [OPTIONS] --input <INPUT>
```

## Description

The `get` command returns the current state of a registry key or value as an instance of the
`Microsoft.Windows/Registry` resource. It expects input as a JSON instance of the resource for the
`--input` option.

The input instance must define the [keyPath][01] property. It uses the `keyPath` value to determine
which registry key to retrieve. If the input instance includes the [valueName][02] property, the
command retrieves the current state of that value instead of the registry key.

## Examples

### Example 1 - Get a registry key

<a id="example-1"></a>

The command returns the current state of the specified registry key as a single line of compressed
JSON without any whitespace.

```powershell
$instance = @{
    keyPath = 'HKLM\Software\Microsoft\Windows NT\CurrentVersion'
} | ConvertTo-Json

registry config get --input $instance
```

```json
{"keyPath":"HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion"}
```

When the specified key doesn't exist, the `_exist` property is `false`.

```powershell
$instance = @{
    keyPath = 'HKCU\Example\Nested\Key'
} | ConvertTo-Json

$instance | registry config get
```

```json
{"keyPath":"HKCU\\Example\\Nested\\Key","_exist":false}
```

### Example 2 - Get a registry value

<a id="example-2"></a>

The command returns the current state of the specified registry value as a single line of compressed
JSON without any whitespace.

```powershell
$instance = @{
    keyPath   = 'HKLM\Software\Microsoft\Windows NT\CurrentVersion'
    valueName = 'SystemRoot'
} | ConvertTo-Json

registry config get --input $instance | ConvertFrom-Json | Format-List
```

```Output
keyPath   : HKLM\Software\Microsoft\Windows NT\CurrentVersion
valueName : SystemRoot
valueData : @{String=C:\WINDOWS}
```

When the specified key doesn't exist, the output only includes the `keyPath` and `_exist`
properties with `_exist` defined as `false`.

```powershell
$instance = @{
    keyPath   = 'HKLM\Software\Microsoft\Windows NT\DoesNotExist'
    valueName = 'SystemRoot'
} | ConvertTo-Json

registry config get --input $instance | ConvertFrom-Json | Format-List
```

```Output
keyPath : HKLM\Software\Microsoft\Windows NT\DoesNotExist
_exist  : False
```

When the specified key exists but the value doesn't, the output only includes the `keyPath`,
`valueName`, and `_exist` properties with `_exist` defined as `false`.

```powershell
$instance = @{
    keyPath   = 'HKLM\Software\Microsoft\Windows NT\CurrentVersion'
    valueName = 'DoesNotExist'
} | ConvertTo-Json

registry config get --input $instance | ConvertFrom-Json | Format-List
```

```Output
keyPath   : HKLM\Software\Microsoft\Windows NT\CurrentVersion
valueName : DoesNotExist
_exist    : False
```

## Options

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the resource instance to retrieve from the system.

The instance must be a string containing a JSON object that is valid for the resource's instance
schema.

```yaml
Type        : string
Mandatory   : true
LongSyntax  : --input <INPUT>
ShortSyntax : -i <INPUT>
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

<!-- Link references -->
[01]: ../../../resources/microsoft/windows/registry/index.md#keypath
[02]: ../../../resources/microsoft/windows/registry/index.md#valuename
