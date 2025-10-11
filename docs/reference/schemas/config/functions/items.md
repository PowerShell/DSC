---
description: Reference for the 'items' DSC configuration document function
ms.date:     10/11/2025
ms.topic:    reference
title:       items
---

## Synopsis

Converts a dictionary object to an array of key-value pairs.

## Syntax

```Syntax
items(inputObject)
```

## Description

The `items()` function converts a dictionary object to an array of key-value pairs.

- Each element in the returned array is an object with two properties: `key` (the
  property name) and `value` (the property value).

This function is useful for iterating over object properties in DSC configurations,
especially when used with loops. It's the companion function to [`toObject()`][03],
which converts an array back to an object.

## Examples

### Example 1 - Convert simple object to array

This example uses [`createObject()`][00] to create a simple object and converts it
to an array of key-value pairs.

```yaml
# items.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[items(createObject('a', 1, 'b', 2, 'c', 3))]"
```

```bash
dsc config get --file items.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - key: a
        value: 1
      - key: b
        value: 2
      - key: c
        value: 3
messages: []
hadErrors: false
```

### Example 2 - Access keys and values

This example shows how to access the keys and values from the items array using array
indexing.

```yaml
# items.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[items(createObject('firstName', 'John', 'lastName', 'Doe'))[0].key]"
```

```bash
dsc config get --file items.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: firstName
messages: []
hadErrors: false
```

### Example 3 - Get length of object

This example shows how to use `items()` with [`length()`][01] to count the number of
properties in an object.

```yaml
# items.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[length(items(createObject('a', 1, 'b', 2, 'c', 3)))]"
```

```bash
dsc config get --file items.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 3
messages: []
hadErrors: false
```

### Example 4 - Handle nested objects

This example demonstrates using `items()` with objects that contain nested objects.
It uses [`length()`][01] and [`copyIndex()`][02] with the `copy` feature to iterate
over each user.

```yaml
# items.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  users:
    type: object
    defaultValue:
      admin:
        name: Administrator
        role: admin
      guest:
        name: Guest User
        role: guest
resources:
- name: "[format('User-{0}', copyIndex())]"
  copy:
    name: userLoop
    count: "[length(items(parameters('users')))]"
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[items(parameters('users'))[copyIndex()].value.name]"
```

```bash
dsc config get --file items.example.4.dsc.config.yaml
```

```yaml
results:
- name: User-0
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Administrator
- name: User-1
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Guest User
messages: []
hadErrors: false
```

## Parameters

### inputObject

The dictionary object to convert to an array of key-value pairs.

```yaml
Type:     object
Required: true
Position: 1
```

## Output

Returns an array where each element is an object with `key` and `value` properties.

```yaml
Type: array
```

## Related functions

- [`createObject()`][00] - Creates an object from key-value pairs
- [`length()`][01] - Returns the number of elements in an array or object
- [`toObject()`][03] - Converts an array of key-value pairs to an object

<!-- Link reference definitions -->
[00]: ./createObject.md
[01]: ./length.md
[02]: ./copyIndex.md
[03]: ./toObject.md
