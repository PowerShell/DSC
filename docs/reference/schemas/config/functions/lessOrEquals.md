---
description: Reference for the 'lessOrEquals' DSC configuration document function
ms.date:     07/24/2025
ms.topic:    reference
title:       lessOrEquals
---

# lessOrEquals

## Synopsis

Checks whether the first value is less than or equal to the second value.

## Syntax

```Syntax
lessOrEquals(<firstValue>, <secondValue>)
```

## Description

The `lessOrEquals()` function checks whether the first value is less than or
equal to the second value, returning `true` if it is and otherwise `false`.
You can use this function to compare two values of the same data type.
If the values are different types, like a string and an integer, DSC returns
an error for this function.

For strings, the comparison is case-sensitive and uses lexicographic ordering
based on character codes.

## Examples

### Example 1 - Compare two numbers

The following example shows how you can use the function to compare two numbers.

```yaml
# lessOrEquals.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare numbers
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: 
      firstLess:        "[lessOrEquals(3, 5)]"
      secondLess:       "[lessOrEquals(5, 3)]"
      equalNumbers:     "[lessOrEquals(5, 5)]"
```

```bash
dsc config get --file lessOrEquals.example.1.dsc.config.yaml
```

```yaml
results:
- name: Compare numbers
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        firstLess: true
        secondLess: false
        equalNumbers: true
messages: []
hadErrors: false
```

### Example 2 - Compare two strings

The following example shows how you can use the function to compare two strings.

```yaml
# lessOrEquals.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      lexicographicLess:    "[lessOrEquals('a', 'b')]"
      lexicographicGreater: "[lessOrEquals('b', 'a')]"
      equalStrings:         "[lessOrEquals('a', 'a')]"
      caseSensitive:        "[lessOrEquals('aa', 'Aa')]"
```

```bash
dsc config get --file lessOrEquals.example.2.dsc.config.yaml
```

```yaml
results:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        lexicographicLess: true
        lexicographicGreater: false
        equalStrings: true
        caseSensitive: false
messages: []
hadErrors: false
```

### Example 3 - Type mismatch error

The following example shows what happens when you try to compare different types.

```yaml
# lessOrEquals.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Type mismatch
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[lessOrEquals(5, 'a')]"
```

```bash
dsc config get --file lessOrEquals.example.3.dsc.config.yaml
```

This will result in an error because you cannot compare a number with a string.

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

The `lessOrEquals()` function expects exactly two input values of the same type.
Separate each value with a comma. If the type of the second input value is different
from the first value, DSC returns an error for the function.

String comparisons are case-sensitive and use lexicographic ordering.

## Output

The `lessOrEquals()` function returns `true` if the first value is less than or
equal to the second value and otherwise `false`.

```yaml
Type: bool
```

<!-- Link reference definitions -->
