---
description: Reference for the 'equals' DSC configuration document function
ms.date:     07/02/2025
ms.topic:    reference
title:       equals
---

# equals

## Synopsis

Checks whether two values are identical.

## Syntax

```Syntax
equals(<inputValue>)
```

## Description

The `equals()` function checks whether two values are identical, returning `true` if they are and
otherwise `false`. You can use this function to compare two values of the same data type. If the
values are different types, like a string and an integer, DSC returns `false` for this function.

## Examples

### Example 1 - Compare two strings

The following example shows how you can use the function to compare two strings.

```yaml
# equals.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: 
      sameCase:         "[equals('a', 'a')]"
      differentCase:    "[equals('a', 'A')]"
      differentLetters: "[equals('a', 'b')]"
```

```bash
dsc config get --file equals.example.1.dsc.config.yaml
```

```yaml
results:
- name: Compare strings
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        sameCase: true
        differentCase: false
        differentLetters: false
messages: []
hadErrors: false
```

### Example 2 - Compare two integers

The following example shows how you can use the function to compare two integers.

```yaml
# equals.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare integers
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      sameInteger:      "[equals(1, 1)]"
      differentInteger: "[equals(1, 2)]"
```

```bash
dsc config get --file equals.example.2.dsc.config.yaml
```

```yaml
results:
- name: Compare integers
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        sameInteger: true
        differentInteger: false
        sameFloat: true
        differentFloat: false
        integerAndFloat: ?
messages: []
hadErrors: false
```

### Example 3 - Compare two arrays

The following example shows how you can use the function to compare two arrays.

```yaml
# equals.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compare arrays
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      sameStringsAndOrder: >-
        [equals(
          createArray('a', 'b', 'c'),
          createArray('a', 'b', 'c')
        )]
      sameStringsDifferentOrder: >-
        [equals(
          createArray('a', 'b', 'c'),
          createArray('c', 'b', 'a')
        )]
      sameNumbers: >-
        [equals(
          createArray(1, 2, 3),
          createArray(1, 2, 3)
        )]
      sameNestedArrays: >-
        [equals(
          createArray(
            createArray('a', 'b', 'c'),
            createArray(1, 2, 3)
          ),
          createArray(
            createArray('a', 'b', 'c'),
            createArray(1, 2, 3)
          )
        )]
```

```bash
dsc config get --file equals.example.3.dsc.config.yaml
```

```yaml
results:
- name: Compare arrays
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        sameStringsAndOrder: true
        sameStringsDifferentOrder: false
        sameNumbers: true
        sameNestedArrays: true
messages: []
hadErrors: false
```

## Parameters

### inputValue

The `equals()` function expects exactly two input values of the same type. Separate each value with
a comma. If the type of the second input value is different from the first value, DSC returns an
error for the function.

String comparisons are case-sensitive. Array comparisons are position-sensitive.

```yaml
Type:         [integer, string, object, array]
Required:     true
MinimumCount: 2
MaximumCount: 2
```

## Output

The `equals()` function returns `true` if the input values are the same and otherwise `false`.

```yaml
Type: bool
```

<!-- Link reference definitions -->
