---
description: Reference for the 'greaterOrEquals' DSC configuration document function
ms.date:     07/24/2025
ms.topic:    reference
title:       greaterOrEquals
---

# greaterOrEquals

## Synopsis

Checks whether the first value is greater than or equal to the second value.

## Syntax

```Syntax
greaterOrEquals(<firstValue>, <secondValue>)
```

## Description

The `greaterOrEquals()` function checks whether the first value is greater
than or equal to the second value, returning `true` if it is and otherwise `false`.
You can use this function to compare two values of the same data type. If the values
are different types, like a string and an integer, DSC returns an error for this function.

For strings, the comparison is case-sensitive and uses lexicographic ordering based on character codes.

## Examples

### Example 1 - Compare two numbers

The following example shows how you can use the function to compare two numbers.

```yaml
# greaterOrEquals.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare numbers
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: 
      firstGreater:     "[greaterOrEquals(5, 3)]"
      secondGreater:    "[greaterOrEquals(3, 5)]"
      equalNumbers:     "[greaterOrEquals(5, 5)]"
```

```bash
dsc config get --file greaterOrEquals.example.1.dsc.config.yaml
```

```yaml
results:
- name: Compare numbers
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        firstGreater: true
        secondGreater: false
        equalNumbers: true
messages: []
hadErrors: false
```

### Example 2 - Compare two strings

The following example shows how you can use the function to compare two strings.

```yaml
# greaterOrEquals.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      lexicographicGreater: "[greaterOrEquals('b', 'a')]"
      lexicographicLess:    "[greaterOrEquals('a', 'b')]"
      equalStrings:         "[greaterOrEquals('a', 'a')]"
      caseSensitive:        "[greaterOrEquals('Aa', 'aa')]"
```

```bash
dsc config get --file greaterOrEquals.example.2.dsc.config.yaml
```

```yaml
results:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        lexicographicGreater: true
        lexicographicLess: false
        equalStrings: true
        caseSensitive: false
messages: []
hadErrors: false
```

### Example 3 - Type mismatch error

The following example shows what happens when you try to compare different types.

```yaml
# greaterOrEquals.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Type mismatch
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[greaterOrEquals('5', 3)]"
```

```bash
dsc config get --file greaterOrEquals.example.3.dsc.config.yaml
```

This will result in an error because you cannot compare a string with a number.

## Parameters

### firstValue

The first value to compare. Must be the same type as the second value.

```yaml
Type:         [number, string]
Required:     true
```

### secondValue

The second value to compare. Must be the same type as the first value.

```yaml
Type:         [number, string]
Required:     true
```

The `greaterOrEquals()` function expects exactly two input values of the same type.
Separate each value with a comma. If the type of the second input value is different
from the first value, DSC returns an error for the function.

String comparisons are case-sensitive and use lexicographic ordering.

## Output

The `greaterOrEquals()` function returns `true` if the first value is greater than
or equal to the second value and otherwise `false`.

```yaml
Type: bool
```

<!-- Link reference definitions -->
