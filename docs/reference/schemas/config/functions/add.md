---
description: Reference for the 'add' DSC configuration document function
ms.date:     03/19/2024
ms.topic:    reference
title:       add
---

# add

## Synopsis

Adds two integers, returning their sum.

## Syntax

```Syntax
add(<operands>)
```

## Description

The `add()` function returns the sum of two integers. It adds the second operand to the first
operand. You can nest calls to `add()` to sum more than two integers.

## Examples

### Example 1 - Add two integers

This example document shows how you can use the `add()` function to return the sum of two integers.

```yaml
# add.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
- name: Sum of 3 and 5
  type: Test/Echo
  properties:
  output: "[add(3, 5)]"
```

```bash
dsc config get --document add.example.1.dsc.config.yaml 
```

```yaml
results:
- name: Sum of 3 and 5
  type: Test/Echo
  result:
    actualState:
      output: 8
messages: []
hadErrors: false
```

### Example 2 - Add output of nested functions

This example document shows how you can use the `add()` function to return the sum of nested
configuration functions that return integer values.

```yaml
# add.example.2.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
- name: Add nested function outputs
  type: Test/Echo
  properties:
    output: "[add(mul(2,3), div(6,3))]"
```

```bash
dsc config get --document add.example.2.dsc.config.yaml
```

```yaml
results:
- name: Add nested function outputs
  type: Test/Echo
  result:
    actualState:
      output: 8
messages: []
hadErrors: false
```

## Parameters

### operands

The `add()` function expects exactly two integers as input. The **operands** can be either an
integer or the output of any configuration function that returns an integer. Separate the
**operands** with a comma (`,`).

```yaml
Type:         integer
Required:     true
MinimumCount: 2
MaximumCount: 2
```

## Output

The `add()` function returns an integer representing the sum of the **operands**.

```yaml
Type: integer
```

<!-- Link reference definitions -->
