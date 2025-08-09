---
description: Reference for the 'length' DSC configuration document function
ms.date:     08/08/2025
ms.topic:    reference
title:       length
---

# length

## Synopsis

Returns the number of elements in an array, properties in an object, or
characters in a string.

## Syntax

```Syntax
length(<value>)
```

## Description

The `length()` function returns the number of elements in a collection or
characters in a string. For arrays, it returns the count of elements. For
objects, it returns the count of properties. For strings, it returns the
count of characters.

## Examples

### Example 1 - Get array length

The following example shows how to get the length of arrays.

```yaml
# length.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  smallArray:
    type: array
    defaultValue:
    - dsc
    - v3
  largeArray:
    type: array
    defaultValue:
    - red
    - green
    - blue
    - yellow
    - purple
  emptyArray:
    type: array
    defaultValue: []
resources:
- name: Check array lengths
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      smallLength: "[length(parameters('smallArray'))]"
      largeLength: "[length(parameters('largeArray'))]"
      emptyLength: "[length(parameters('emptyArray'))]"
```

```bash
dsc config get --file length.example.1.dsc.config.yaml
```

```yaml
results:
- name: Check array lengths
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        smallLength: 2
        largeLength: 5
        emptyLength: 0
messages: []
hadErrors: false
```

### Example 2 - Get object property count

The following example shows how to get the number of properties in objects.

```yaml
# length.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  userProfile:
    type: object
    defaultValue:
      firstName: John
      lastName: Doe
      email: john.doe@example.com
      age: 30
  emptyConfig:
    type: object
    defaultValue: {}
resources:
- name: Check object property counts
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      profileProperties: "[length(parameters('userProfile'))]"
      emptyProperties: "[length(parameters('emptyConfig'))]"
```

```bash
dsc config get --file length.example.2.dsc.config.yaml
```

```yaml
results:
- name: Check object property counts
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        profileProperties: 4
        emptyProperties: 0
messages: []
hadErrors: false
```

### Example 3 - Get string character count

The following example shows how to get the length of strings.

```yaml
# length.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  message:
    type: string
    defaultValue: "Hello DSC!"
  longText:
    type: string
    defaultValue: "This is a longer string with more characters to demonstrate length calculation."
resources:
- name: Check string lengths
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      messageLength: "[length(parameters('message'))]"
      longTextLength: "[length(parameters('longText'))]"
      emptyStringLength: "[length('')]"
      literalLength: "[length('DSC')]"
```

```bash
dsc config get --file length.example.3.dsc.config.yaml
```

```yaml
results:
- name: Check string lengths
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        messageLength: 10
        longTextLength: 78
        emptyStringLength: 0
        literalLength: 3
messages: []
hadErrors: false
```

## Parameters

### value

The value to get the length of.

```yaml
Type:         [array, object, string]
Required:     true
```

The `length()` function expects exactly one input value of type array, object,
or string.

## Output

The `length()` function returns an integer representing the count of elements,
properties, or characters.

```yaml
Type: number
```

<!-- Link reference definitions -->
