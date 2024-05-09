---
description: Reference for the 'min' DSC configuration document function
ms.date:     04/09/2024
ms.topic:    reference
title:       min
---

# min

## Synopsis

Returns the minimum value from an array of integers or a comma-separated list of integers.

## Syntax

```Syntax
min(<integerList>)
```

## Description

The `min` function returns the minimum value from an array of integers or a comma-separated list of
integers.

## Examples

### Example 1 - Return minimum from a comma-separated list of integers

This configuration returns the smallest number from a list of integers.

```yaml
# min.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Echo minimum value
  type: Test/Echo
  properties:
    output: "[min(3, 2, 5, 1, 7)]"
```

```bash
dsc config get --document min.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Echo minimum value
  type: Test/Echo
  result:
    actualState:
      output: 1
messages: []
hadErrors: false
```

### Example 2 - Return minimum from an array of integers

This configuration echoes the smallest number from an array of integers that is retrieved as a
[reference][01] to another resource instance. It uses YAML's folded multiline syntax to make the
function more readable.

```yaml
# min.example.2.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Echo integer array
  type: Test/Echo
  properties:
    output:
    - 3
    - 2
    - 5
    - 1
    - 7
- name: Echo minimum integer
  type: Test/Echo
  properties:
    output: >-
      [min(
        reference(
          resourceId('Test/Echo', 'Echo integer array')
        ).actualState.output
      )]
  dependsOn:
  - "[resourceId('Test/Echo', 'Echo integer array')]"
```

```bash
dsc config get --document min.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo integer array
  type: Test/Echo
  result:
    actualState:
      output:
      - 3
      - 2
      - 5
      - 1
      - 7
- name: Echo minimum integer
  type: Test/Echo
  result:
    actualState:
      output: 1
```

## Parameters

### integerList

The `min()` function expects either a single array of integers or a comma-separated array of
integers. When you pass integers directly, separate each integer with a comma. When you pass an
array object, the function only takes a single array as an argument. You can use the
[createArray()][02] function to combine multiple arrays or an array and additional integers.

```yaml
Type:         [integer, array(integer)]
Required:     true
MinimumCount: 1
MaximumCount: 18446744073709551615
```

## Output

The `min()` function returns a single integer representing the smallest value in the input.

```yaml
Type: integer
```

<!-- Link reference definitions -->
[01]: ./reference.md
[02]: ./createArray.md
