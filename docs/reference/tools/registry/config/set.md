---
description: Command line reference for the 'registry config set' command
ms.date:     03/18/2025
ms.topic:    reference
title:       registry config set
---

# registry config set

## Synopsis

Apply registry configuration.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry config config set [OPTIONS] --input <INPUT>
```

## Description

The `set` command returns the current state of a registry key or value as an instance of the
`Microsoft.Windows/Registry` resource. It expects input as a JSON instance of the resource for the
`--input` option.

The input instance must define the [keyPath][01] property. It uses the `keyPath` value to determine
which registry key to configure. If the input instance includes the [valueName][02] property, the
command configures the current state of that value instead of the registry key.

This command can only create and modify registry keys and values. To delete a registry key or
value, use the [registry config delete][03] command.

For more information about the available properties for configuring registry keys and values, see
[Microsoft.Windows/Registry][04].

## Examples

### Example 1 - Ensure a registry key exists

<a id="example-1"></a>

Because the input instance defines the `_exist` property as `true`,the command creates the
`HKCU\ExampleKey` if it doesn't exist.

```powershell
$InputInstance = @{
    _exist = $true
    keyPath = 'HKCU\Example\Key'
} | ConvertTo-Json

registry config get --input $InputInstance | ConvertFrom-Json | Format-List
registry config set --input $InputInstance | ConvertFrom-Json | Format-List
registry config get --input $InputInstance | ConvertFrom-Json | Format-List
```

```Output
keyPath : HKCU\Example\Key
_exist  : False



keyPath : HKCU\Example\Key
```

### Example 2 - Ensure a registry value exists

<a id="example-2"></a>

The instance combines the values of the `keyPath`, `valueName`, and `valueData` properties to
ensure that the `ExampleKey` registry key in the current user hive has a value named
`ExampleValue`. If the value doesn't already exist, the command creates the value with the string
`SomeValue` as its data.

```powershell
$InputInstance = @{
    _exist    = $true
    keyPath   = 'HKCU\Example\Key'
    valueName = 'ExampleValue'
    valueData = @{
        String = 'SomeValue'
    }
} | ConvertTo-Json

registry config get --input $InputInstance | ConvertFrom-Json | Format-List
registry config set --input $InputInstance | ConvertFrom-Json | Format-List
registry config get --input $InputInstance | ConvertFrom-Json | Format-List
```

```Output
keyPath   : HKCU\ExampleKey
valueName : ExampleValue
_exist    : False



keyPath   : HKCU\Example\Key
valueName : ExampleValue
valueData : @{String=SomeValue}
```

## Options

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the desired state of the resource instance to enforce on the system.

The instance must be a string containing a JSON object that is valid for
the resource's instance schema.

```yaml
Type        : string
Mandatory   : true
LongSyntax  : --input <INPUT>
ShortSyntax : -i <INPUT>
```

### -w, --what-if

<a id="-w"></a>
<a id="--what-if"></a>

When you specify this flag option, the command doesn't actually change the system state. Instead,
it returns JSON messages to stderr indicating _how_ the operation will change system state when
called without this option. This option enables the resource to support the what-if mode for
**Set** operations in DSC.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --what-if
ShortSyntax : -w
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
[03]: ./delete.md
[04]: ../../../resources/microsoft/windows/registry/index.md
