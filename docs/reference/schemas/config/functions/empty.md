---
description: Reference for the 'empty' DSC configuration document function
ms.date:     08/08/2025
ms.topic:    reference
title:       empty
---

# empty

## Synopsis

Checks whether a value is empty.

## Syntax

```Syntax
empty(<value>)
```

## Description

The `empty()` function checks whether a value is empty, returning `true` if
it is and `false` otherwise. For arrays and objects, it returns `true` if
they contain no elements or properties. For strings, it returns `true` if
the string is empty (zero length).

## Examples

### Example 1 - Check empty arrays

The following example shows how to check if arrays are empty.

```yaml
# empty.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  populatedArray:
    type: array
    defaultValue:
    - item1
    - item2
    - item3
  emptyArray:
    type: array
    defaultValue: []
resources:
- name: Check array emptiness
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      populatedEmpty: "[empty(parameters('populatedArray'))]"
      emptyArrayEmpty: "[empty(parameters('emptyArray'))]"
```

```bash
dsc config get --file empty.example.1.dsc.config.yaml
```

```yaml
results:
- name: Check array emptiness
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        populatedEmpty: false
        emptyArrayEmpty: true
messages: []
hadErrors: false
```

### Example 2 - Check empty objects

The following example shows how to check if objects are empty.

```yaml
# empty.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  populatedObject:
    type: object
    defaultValue:
      name: John
      age: 30
  emptyObject:
    type: object
    defaultValue: {}
resources:
- name: Check object emptiness
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      populatedEmpty: "[empty(parameters('populatedObject'))]"
      emptyObjectEmpty: "[empty(parameters('emptyObject'))]"
```

```bash
dsc config get --file empty.example.2.dsc.config.yaml
```

```yaml
results:
- name: Check object emptiness
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        populatedEmpty: false
        emptyObjectEmpty: true
messages: []
hadErrors: false
```

### Example 3 - Check empty strings

The following example shows how to check if strings are empty.

```yaml
# empty.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  populatedString:
    type: string
    defaultValue: "Hello World"
  emptyString:
    type: string
    defaultValue: ""
resources:
- name: Check string emptiness
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      populatedEmpty: "[empty(parameters('populatedString'))]"
      emptyStringEmpty: "[empty(parameters('emptyString'))]"
      literalEmpty: "[empty('')]"
```

```bash
dsc config get --file empty.example.3.dsc.config.yaml
```

```yaml
results:
- name: Check string emptiness
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        populatedEmpty: false
        emptyStringEmpty: true
        literalEmpty: true
messages: []
hadErrors: false
```

### Example 4 - Conditional resource deployment

The following example shows a practical use case for checking if a
configuration array is empty before deploying resources.

```yaml
# empty.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  serverList:
    type: array
    defaultValue: []
resources:
- name: Check if servers configured
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      message: "[if(empty(parameters('serverList')), 'No servers to configure', concat('Configuring ', string(length(parameters('serverList'))), ' servers'))]"
```

```bash
dsc config get --file empty.example.4.dsc.config.yaml
```

```yaml
results:
- name: Check if servers configured
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        message: "No servers to configure"
messages: []
hadErrors: false
```

## Parameters

### value

The value to check for emptiness.

```yaml
Type:         [array, object, string]
Required:     true
```

The `empty()` function expects exactly one input value of type array, object,
or string.

## Output

The `empty()` function returns `true` if the value is empty and `false`
otherwise.

```yaml
Type: bool
```

<!-- Link reference definitions -->
