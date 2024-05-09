---
description: Reference for the 'mod' DSC configuration document function
ms.date:     03/19/2024
ms.topic:    reference
title:       mod
---

# mod

## Synopsis

Returns the remainder for the division of two numbers.

## Syntax

```Syntax
mod(<operands>)
```

## Description

The `mod()` function returns the remainder for the division of two integers.

## Examples

### Example 1 - Get the remainder for two integers

This example document shows how you can use the `mod()` function to return the remainder of a
division for two integers.

```yaml
# mod.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Remainder for integers
  type: Test/Echo
  properties:
  output: "[mod(7, 5)]"
```

```bash
dsc config get --document mod.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Remainder for integers
  type: Test/Echo
  result:
    actualState:
      output: 2
messages: []
hadErrors: false
```

### Example 2 - Get the remainder for output of nested functions

This configuration document uses the `mod()` function to get the remainder for diving the output of
two other mathematical operations.

```yaml
# mod.example.2.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Remainder for nested functions
  type: Test/Echo
  properties:
    output: "[mod(add(9, 5), mul(6, 2))]"
```

```bash
dsc config get --document mod.example.2.dsc.config.yaml
```

```yaml
results:
- name: Remainder for nested functions
  type: Test/Echo
  result:
    actualState:
      output: 2
messages: []
hadErrors: false
```

## Parameters

### operands

The `mod()` function expects exactly two integers as input. The **operands** can be either an
integer or the output of any configuration function that returns an integer. The function divides
the first operand by the second operand. Separate the **operands** with a comma (`,`).

```yaml
Type:         integer
Required:     true
MinimumCount: 2
MaximumCount: 2
```

## Output

The function returns an integer representing the remainder of the division operation for the
**operands**.

```yaml
Type: integer
```

<!-- Link reference definitions -->
