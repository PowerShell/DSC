---
description: Reference for the 'less' DSC configuration document function
ms.date:     07/24/2025
ms.topic:    reference
title:       less
---

# less

## Synopsis

Checks whether the first value is less than the second value.

## Syntax

```Syntax
less(<firstValue>, <secondValue>)
```

## Description

The `less()` function checks whether the first value is less than the second value,
returning `true` if it is and otherwise `false`. You can use this function to compare
two values of the same data type. If the values are different types, like a string and
an integer, DSC returns an error for this function.

For strings, the comparison is case-sensitive and uses lexicographic ordering based on character codes.

## Examples

### Example 1 - Compare two numbers

The following example shows how you can use the function to compare two numbers.

```yaml
# less.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare numbers
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: 
      firstLess:        "[less(3, 5)]"
      secondLess:       "[less(5, 3)]"
      equalNumbers:     "[less(5, 5)]"
```

```bash
dsc config get --file less.example.1.dsc.config.yaml
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
        equalNumbers: false
messages: []
hadErrors: false
```

### Example 2 - Compare two strings

The following example shows how you can use the function to compare two strings.

```yaml
# less.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      lexicographicLess:    "[less('a', 'b')]"
      lexicographicGreater: "[less('b', 'a')]"
      caseSensitive:        "[less('A', 'a')]"
```

```bash
dsc config get --file less.example.2.dsc.config.yaml
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
        caseSensitive: true
messages: []
hadErrors: false
```

### Example 3 - Type mismatch error

The following example shows what happens when you try to compare different types.

```yaml
# less.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Type mismatch
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[less(1, 'b')]"
```

```bash
dsc config get --file less.example.3.dsc.config.yaml
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

The `less()` function expects exactly two input values of the same type.
Separate each value with a comma. If the type of the second input value is
different from the first value, DSC returns an error for the function.

String comparisons are case-sensitive and use lexicographic ordering.

## Output

The `less()` function returns `true` if the first value is less than the second
value and otherwise `false`.

```yaml
Type: bool
```

<!-- Link reference definitions -->
