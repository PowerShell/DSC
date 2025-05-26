---
description: Reference for the 'max' DSC configuration document function
ms.date:     02/28/2025
ms.topic:    reference
title:       max
---

# max

## Synopsis

Returns the maximum value from a set of integers.

## Syntax

```Syntax
max(<integerList>)
```

## Description

The `max()` function returns the maximum value from an array of integers or a comma-separated list
of integers.

## Examples

### Example 1 - Return maximum from a comma-separated list of integers

This configuration returns the largest number from a list of integers.

```yaml
# max.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo maximum value
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[max(3, 2, 5, 1, 7)]"
```

```bash
dsc config get --document max.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Echo maximum value
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 7
messages: []
hadErrors: false
```

### Example 2 - Return maximum from an array of integers

This configuration echoes the largest number from an array of integers that is retrieved as a
[reference][01] to another resource instance. It uses YAML's folded multiline syntax to make the
function more readable.

```yaml
# max.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo integer array
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
    - 3
    - 2
    - 5
    - 1
    - 7
- name: Echo maximum integer
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [max(
        reference(
          resourceId('Microsoft.DSC.Debug/Echo', 'Echo integer array')
        ).actualState.output
      )]
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'Echo integer array')]"
```

```bash
dsc config get --document max.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo integer array
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - 3
      - 2
      - 5
      - 1
      - 7
- name: Echo maximum integer
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 7
```

## Parameters

### integerList

The `max()` function expects either a single array of integers or a comma-separated array of
integers. When you pass integers directly, separate each integer with a comma. When you pass an
array object, the function only takes a single array as an argument.

```yaml
Type:         [integer, array(integer)]
Required:     true
MinimumCount: 1
MaximumCount: 18446744073709551615
```

## Output

The `max()` function returns a single integer representing the largest value in the input.

```yaml
Type: integer
```

<!-- Link reference definitions -->
[01]: ./reference.md
