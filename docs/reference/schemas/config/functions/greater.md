---
description: Reference for the 'greater' DSC configuration document function
ms.date:     07/24/2025
ms.topic:    reference
title:       greater
---

# greater

## Synopsis

Checks whether the first value is greater than the second value.

## Syntax

```Syntax
greater(<firstValue>, <secondValue>)
```

## Description

The `greater()` function checks whether the first value is greater than the second value,
returning `true` if it is and otherwise `false`. You can use this function to compare two
values of the same data type. If the values are different types, like a string and an
integer, DSC returns an error for this function.

For strings, the comparison is case-sensitive and uses lexicographic ordering based on character codes.

## Examples

### Example 1 - Compare two numbers

The following example shows how you can use the function to compare two numbers.

```yaml
# greater.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare numbers
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: 
      firstGreater:     "[greater(5, 3)]"
      secondGreater:    "[greater(3, 5)]"
      equalNumbers:     "[greater(5, 5)]"
```

```bash
dsc config get --file greater.example.1.dsc.config.yaml
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
        equalNumbers: false
messages: []
hadErrors: false
```

### Example 2 - Compare two strings

The following example shows how you can use the function to compare two strings.

```yaml
# greater.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      lexicographicGreater: "[greater('b', 'a')]"
      lexicographicLess:    "[greater('a', 'b')]"
      caseSensitive:        "[greater('a', 'A')]"
```

```bash
dsc config get --file greater.example.2.dsc.config.yaml
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
        caseSensitive: true
messages: []
hadErrors: false
```

### Example 3 - Type mismatch error

The following example shows what happens when you try to compare different types.

```yaml
# greater.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Type mismatch
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[greater('5', 3)]"
```

```bash
dsc config get --file greater.example.3.dsc.config.yaml
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

The `greater()` function expects exactly two input values of the same type.
Separate each value with a comma. If the type of the second input value is
different from the first value, DSC returns an error for the function.

String comparisons are case-sensitive and use lexicographic ordering.

## Output

The `greater()` function returns `true` if the first value is greater than
the second value and otherwise `false`.

```yaml
Type: bool
```

<!-- Link reference definitions -->
