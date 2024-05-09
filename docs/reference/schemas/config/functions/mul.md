---
description: Reference for the 'mul' DSC configuration document function
ms.date:     03/20/2024
ms.topic:    reference
title:       mul
---

# mul

## Synopsis

Returns the product of multiplying two integers.

## Syntax

```Syntax
mul(<operands>)
```

## Description

The `mul()` function returns the product of multiplying two integers. It multiplies the first operand
by the second operand. You can nest calls to `mul()` to multiply more than two integers.

## Examples

### Example 1 - Multiply two integers

This example document multiplies two integers to return a product for the output.

```yaml
# mul.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Multiplying integers
  type: Test/Echo
  properties:
  output: "[mul(3, 5)]"
```

```bash
dsc config get --document mul.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Multiplying integers
  type: Test/Echo
  result:
    actualState:
      output: 15
messages: []
hadErrors: false
```

### Example 2 - Multiply output of nested functions

This document shows how you can multiply the output of nested configuration functions.

```yaml
# mul.example.2.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Multiplying nested function outputs
  type: Test/Echo
  properties:
    output: "[mul(add(3, 2), div(12, 4))]"
```

```bash
dsc config get --document mul.example.2.dsc.config.yaml
```

```yaml
results:
- name: Multiplying nested function outputs
  type: Test/Echo
  result:
    actualState:
      output: 15
messages: []
hadErrors: false
```

## Parameters

### operands

The `mul()` function expects exactly two integers as input. The **operands** can be either an integer
or the output of any configuration function that returns an integer. The function divides the first
operand by the second operand. Separate the **operands** with a comma (`,`).

```yaml
Type:         integer
Required:     true
MinimumCount: 2
MaximumCount: 2
```

## Output

The `mul()` function returns an integer representing the product of the multiplied **operands**.

```yaml
Type: integer
```

<!-- Link reference definitions -->
