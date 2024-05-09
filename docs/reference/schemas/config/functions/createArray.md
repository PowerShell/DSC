---
description: Reference for the 'createArray' DSC configuration document function
ms.date:     04/09/2024
ms.topic:    reference
title:       createArray
---

# createArray

## Synopsis

Returns an array of values from input.

## Syntax

```Syntax
createArray(<inputValue>)
```

## Description

The `createArray()` function returns an array of values from the input values. You can use this
function to create arrays of any type. The input values must be of the same type - numbers,
strings, objects, or arrays. When the input values are objects or arrays, they do not need be
objects with the same properties or arrays of the same type. When the input values are arrays, the
function returns an array of arrays.

## Examples

### Example 1 - Create an array of integers

example synopsis

```yaml
# createArray.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Echo array of integers
  type: Test/Echo
  properties:
    output: "[createArray(1, 3, 5)]"
```

```bash
dsc config get --document createArray.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Echo array of integers
  type: Test/Echo
  result:
    actualState:
      output:
      - 1
      - 3
      - 5
messages: []
hadErrors: false
```

### Example 2 - Create an array of arrays

This configuration returns an array where the items in the array are also arrays. The first
sub-array contains only integers. The second sub-array contains only strings.

```yaml
# createArray.example.2.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Create array of arrays
  type: Test/Echo
  properties:
    output: "[createArray(createArray(1,3,5), createArray('a', 'b', 'c'))]"
```

```bash
dsc config get --document createArray.example.2.dsc.config.yaml
```

```yaml
results:
- name: Create array of arrays
  type: Test/Echo
  result:
    actualState:
      output:
      - - 1
        - 3
        - 5
      - - a
        - b
        - c
messages: []
hadErrors: false
```

### Example 3 - Create a flattened array of strings

This configuration uses the [concat()][01] function to concatenate two newly created arrays of
strings. It uses YAML's folded multiline string syntax to make the function more readable.

```yaml
# createArray.example.3.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Echo flattened array
  type: Test/Echo
  properties:
    output: >-
      [concat(
        createArray('a', 'b', 'c'),
        createArray('d', 'e', 'f')
      )]
```

```bash
dsc config get --document createArray.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo flattened array
  type: Test/Echo
  result:
    actualState:
      output:
      - a
      - b
      - c
      - d
      - e
      - f
messages: []
hadErrors: false
```

## Parameters

### inputValue

The `createArray()` function expects zero or more input values of the same type. Separate each
value with a comma. If the type of any input value is different from the first value, DSC returns
an error for the function.

```yaml
Type:         [integer, string, number, object, array]
Required:     false
MinimumCount: 0
MaximumCount: 18446744073709551615
```

## Output

The `createArray()` function returns an array of values. When the input values are arrays, the
returned value is an array of arrays, not a flattened array of the input values. You can return a
flattened array of string arrays with the [concat()][01] function, as in
[example 3](#example-3---create-a-flattened-array-of-strings).

```yaml
Type: array
```

<!-- Link reference definitions -->
[01]: ./concat.md
