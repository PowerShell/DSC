---
description: Reference for the 'concat' DSC configuration document function
ms.date:     02/28/2025
ms.topic:    reference
title:       concat
---

# concat

## Synopsis

Returns a string of combined values.

## Syntax

```Syntax
concat(<inputValue>, <inputValue>[, <inputValue>...])
```

## Description

The `concat()` function combines multiple values and returns the concatenated values as a single
string. Separate each value with a comma. The `concat()` function is variadic. You must pass at
least two values to the function. The function can accept any number of arguments.

The function concatenates the input values without any joining character. It accepts only strings
or arrays of strings as input values. The input values must be of the same type. If you pass a
string and an array to the same function, the function raises an error.

## Examples

### Example 1 - Concatenate strings

The configuration uses the `concat()` function to join the strings `abc` and `def`

```yaml
# concat.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo 'abcdef'
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[concat('abc', 'def')]"
```

```bash
dsc --input-file concat.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Echo 'abcdef'
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: abcdef
messages: []
hadErrors: false
```

### Example 2 - Concatenate arrays of strings

The configuration uses the `concat()` function to return a combined array of strings from two arrays of strings. It uses YAML's folded multiline syntax to make the function more readable.

```yaml
# concat.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo ['a', 'b', 'c', 'd', 'e', 'f']
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [concat(
        createArray('a', 'b', 'c'),
        createArray('d', 'e', 'f')
      )]
```

```bash
dsc config get --document concat.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo ['a', 'b', 'c', 'd', 'e', 'f']
  type: Microsoft.DSC.Debug/Echo
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

The `concat()` function expects two or more input values of the same type to concatenate. Each
value must be either a string or an array of strings. If one value is a string and the other an
array, or either value isn't a string or array of strings, DSC raises an error when validating the
configuration document.

```yaml
Type:         [string, array(string)]
Required:     true
MinimumCount: 2
MaximumCount: 18446744073709551615
```

## Output

When every **inputValue** is a string, `concat()`returns a single string with every **inputValue**
concatenated together. When every **inputValue** is an array of strings, `concat()` returns a
flattened array containing the strings from each input array.

```yaml
Type: [string, array]
```

<!-- Link reference definitions -->
