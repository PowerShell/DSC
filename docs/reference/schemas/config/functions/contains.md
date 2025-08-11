---
description: Reference for the 'contains' DSC configuration document function
ms.date:     08/08/2025
ms.topic:    reference
title:       contains
---

# contains

## Synopsis

Checks whether a collection contains a specific value or whether a string
contains a substring.

## Syntax

```Syntax
contains(<collection>, <value>)
```

## Description

The `contains()` function checks whether a collection (array, object, or
string) contains a specific value, returning `true` if it does and `false`
otherwise. For arrays, it checks if the value exists as an element. For
objects, it checks if the value exists as a property key or value. For
strings, it checks if the value exists as a substring.

The function accepts string and number values for the search parameter when
used with arrays, objects, or strings.

## Examples

### Example 1 - Check array for values

The following example shows how to check if an array contains specific values.

```yaml
# contains.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  myArray:
    type: array
    defaultValue:
    - apple
    - banana
    - 42
    - true
resources:
- name: Check array contents
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      hasApple:    "[contains(parameters('myArray'), 'apple')]"
      hasOrange:   "[contains(parameters('myArray'), 'orange')]"
      hasNumber42: "[contains(parameters('myArray'), 42)]"
      hasNumber99: "[contains(parameters('myArray'), 99)]"
```

```bash
dsc config get --file contains.example.1.dsc.config.yaml
```

```yaml
results:
- name: Check array contents
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hasApple: true
        hasOrange: false
        hasNumber42: true
        hasNumber99: false
messages: []
hadErrors: false
```

### Example 2 - Check object for keys and values

The following example shows how to check if an object contains specific keys
or values.

```yaml
# contains.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  myObject:
    type: object
    defaultValue:
      name: John
      age: 30
      city: Seattle
resources:
- name: Check object contents
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      hasNameKey:      "[contains(parameters('myObject'), 'name')]"
      hasEmailKey:     "[contains(parameters('myObject'), 'email')]"
      hasSeattleValue: "[contains(parameters('myObject').city, 'Seattle')]"
      hasAge30Value:   "[contains(parameters('myObject').age, 30)]"
```

```bash
dsc config get --file contains.example.2.dsc.config.yaml
```

```yaml
results:
- name: Check object contents
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hasNameKey: true
        hasEmailKey: false
        hasSeattleValue: true
        hasAge30Value: true
messages: []
hadErrors: false
```

### Example 3 - Check string for substrings

The following example shows how to check if a string contains specific
substrings.

```yaml
# contains.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  myString:
    type: string
    defaultValue: "Hello DSC 123"
resources:
- name: Check string contents
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      hasHello:  "[contains(parameters('myString'), 'Hello')]"
      hasDSC:  "[contains(parameters('myString'), 'DSC')]"
      hasNumber: "[contains(parameters('myString'), '123')]"
      hasXYZ:    "[contains(parameters('myString'), 'XYZ')]"
```

```bash
dsc config get --file contains.example.3.dsc.config.yaml
```

```yaml
results:
- name: Check string contents
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hasHello: true
        hasDSC: true
        hasNumber: true
        hasXYZ: false
messages: []
hadErrors: false
```

## Parameters

### collection

The collection to search in (array, object, or string).

```yaml
Type:         [array, object, string]
Required:     true
```

### value

The value to search for. Must be a string or number.

```yaml
Type:         [string, number]
Required:     true
```

The `contains()` function expects exactly two input values. The first
parameter is the collection to search, and the second is the value to find.
Complex objects and arrays cannot be used as search values.

## Output

The `contains()` function returns `true` if the collection contains the
specified value and `false` otherwise.

```yaml
Type: bool
```

<!-- Link reference definitions -->
