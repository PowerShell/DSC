---
description: Command line reference for the 'registry config delete' command
ms.date:     03/25/2025
ms.topic:    reference
title:       registry config delete
---

# registry config delete

## Synopsis

Retrieve registry configuration.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry config delete  [OPTIONS] --input <INPUT>
```

## Description

The `delete` command removes a registry key or value as an instance of the
`Microsoft.Windows/Registry` resource. It expects input as a JSON instance of the resource for the
`--input` option.

The input instance must define the [keyPath][01] property. It uses the `keyPath` value to determine
which registry key to operate on. If the input instance includes the [valueName][02] property, the
command remove that value instead of the registry key itself.

## Examples

### Example 1 - delete a registry value

<a id="example-1"></a>

The command returns the current state of the specified registry value as a single line of compressed
JSON without any whitespace.

```powershell
$instance = @{
    keyPath   = 'HKCU\Example\Key'
    valueName = 'ExampleValue'
} | ConvertTo-Json

registry config delete --input $instance | ConvertFrom-Json | Format-List
```

### Example 2 - delete a registry key

<a id="example-2"></a>

The command returns the current state of the specified registry key as a single line of compressed
JSON without any whitespace.

```powershell
$instance = @{
    _exist = $true
    keyPath = 'HKCU\Example\Key'
} | ConvertTo-Json

registry config delete --input $instance | ConvertFrom-Json | Format-List
```

## Options

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the resource instance to remove from the system.

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
[01]: ../../../resources/Microsoft/Windows/Registry/index.md#keypath
[02]: ../../../resources/Microsoft/Windows/Registry/index.md#valuename
