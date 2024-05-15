---
description: JSON schema reference for valid parameter data types in a configuration document.
ms.date:     01/17/2024
ms.topic:    reference
title:       DSC configuration parameter data type schema reference
---

# DSC configuration parameter data type schema reference

## Synopsis

Defines valid data types for a DSC configuration parameter

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/definitions/parameters/dataTypes.json
Type:          string
ValidValues:   [array, bool, int, object, string, secureobject, securestring]
```

## Description

Parameters in a [DSC Configuration document][01] must be a valid data type.

The valid data types for a parameter are:

- `array` for arrays
- `bool` for booleans
- `int` for integers
- `object` for objects
- `string` for strings
- `secureobject` for secure objects
- `securestring` for secure strings

Access parameters in a configuration using this syntax:

```yaml
"[parameter('<parameter-name>')]"
```

In YAML, the parameter syntax needs to be enclosed in double-quotes when used as an inline value.
If the syntax isn't quoted, YAML interprets the syntax as an array.

```yaml
valid:  "[parameter('example')]"
# This unquoted syntax:
invalid: [parameter('example')]
# Evaluates to this YAML:
invalid:
  - parameter('example')
```

## Arrays

Arrays are a list of one or more values. The values in the array can be any valid data type. Values
in the array can be the same type or different types.

```yaml
parameters:
  exampleIntArray:
    type: array
    defaultValue:
      - 1
      - 2
      - 3
  exampleMixedArray:
    type: array
    defaultValue:
      - 1
      - true
      - example string
```

Access items in an array can by their index. The first item in an array is index `0`. Use this
syntax to access an index in an array parameter:

```yaml
"[parameter('<parameter-name>')[<index>]]"
```

```yaml
parameters:
  members:
    type: array
    defaultValue:
      - first
      - second
      - third
resources:
  # Use the entire array as the value for a resource property
  - name: Operators Group
    type: Example.Security/Group
    properties:
      groupName: operators
      members: "[parameter('members')]"
  # Use a single item in the array as the value for a resource property
  - name: Admin Group
    type: Example.Security/Group
    properties:
      groupName: admins
      members:
        - "[parameter('members')[0]]"
```

## Booleans

Boolean values are either `true` or `false`.

```yaml
parameters:
  exampleBool:
    type:         bool
    defaultValue: true
```

## Integers

Integer values are numbers without a fractional part. Integer values may be limited by integrating
tools or the DSC Resources they're used with. DSC itself supports integer values between
`-9223372036854775808` and `9223372036854775807`.

```yaml
parameters:
  exampleInt:
    type:         int
    defaultValue: 12
```

## Objects

Objects define a set of key-value pairs. The value for each key can be any valid data type. The
values can be the same type or different types.

```yaml
parameters:
  exampleObject:
    type: object
    defaultValue:
      scope:               machine
      updateAutomatically: true
      updatefrequency:     30
```

Access keys in the object using dot-notation. Dot-notation uses this syntax:

```yaml
"[parameters('<parameter-name>').<key-name>]
```

```yaml
parameters:
  tstoy:
    type: object
    defaultValue:
      scope:               machine
      updateAutomatically: true
      updatefrequency:     30
  registryKeys:
    type: object
    defaultValue:
      productName:
        keyPath:   HKLM\Software\Microsoft\Windows NT\CurrentVersion
        valueName: ProductName
      systemRoot:
        keyPath:   HKLM\Software\Microsoft\Windows NT\CurrentVersion
        valueName: SystemRoot
resources:
  # Use the base object for the property definition
  - name: TSToy
    type: TSToy.Example/gotstoy
    properties: "[parameter('tstoy')]"
  # Use dot-notation for the property definition
  - name: Windows Product Name
    type: Microsoft.Windows/Registry
    properties: "[parameter('registryKeys').productName]"
  # Use dot-notation for each value in the property definition
  - name: Windows System Root
    type: Microsoft.Windows/Registry
    properties:
      keyPath:   "[parameters('registryKeys').systemRoot.keyPath]"
      valueName: "[parameters('registryKeys').systemRoot.valueName]"
```

## Strings

Strings are an arbitrary set of text.

```yaml
parameters:
  exampleString:
    type: string
    defaultValue: This example includes spaces and 'quoted' "text."
```

To define a long string without newlines in YAML, use the folded block syntax by adding a `>` and a
line break after the key. Then, indent the next line. Every line in the string must start at the
same level of indentation. The lines are combined with a single space instead of newlines. To trim
trailing whitespace, use `>-` instead of `>`.

```yaml
parameters:
  foldedBlockExample:
    type: string
    defaultValue: >-
      This example spans multiple lines
      in the definition, but the lines are
      joined with spaces instead of newlines.
```

To define a long string with newlines in YAML, use the literal block syntax by adding a `|` and a
line break after the key. Then, indent the next line. Every line in the string must start at the
same level of indentation or higher. The lines are interpreted literally, but with the leading
indentation removed. If any lines after the first line are indented more than the first line, only
the extra indentation is preserved. To trim trailing whitespace, use `|-` instead of `|`.

```yaml
parameters:
  literalBlockExample:
    type: string
    defaultValue: |-
      This example spans multiple lines
      in the definition.

      It can even include paragraphs.

        When a line is indented further
        than the first line, that extra
        indentation is preserved.
```

## Secure strings and objects

Secure strings use the same format as strings and secure objects use the same format as objects.
The `secure*` data types indicate that DSC and integrating tools shouldn't log or record the
values. If a secure data type parameter is used for a resource instance property that doesn't
expect a secure value, the resource may still log or record the value. If the resource has
independent logging or recording that isn't handled by DSC, the value may be stored insecurely.

Use secure strings for passwords and secrets. Never define a default value for secure string or
secure object parameters.

```yaml
parameters:
  password:
    type: securestring
  sensitiveOptions:
    type: secureobject
```

[01]: ../../config/document.md
