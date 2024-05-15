---
description: Reference for the 'sub' DSC configuration document function
ms.date:     03/20/2024
ms.topic:    reference
title:       sub
---

# sub

## Synopsis

Returns the difference of two integers.

## Syntax

```Syntax
sub(<operands>)
```

## Description

The `sub()` function returns the difference of two integers. It subtracts the second operand from the
first operand. You can nest calls to `sub()` to subtract more than two integers.

## Examples

### Example 1 - Subtract two integers

This example document shows how you can use the `sub()` function to return the difference of two
integers.

```yaml
# sub.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Subtract integers
  type: Test/Echo
  properties:
    output: "[sub(7, 3)]"
```

```bash
dsc config get --document sub.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Subtract integers
  type: Test/Echo
  result:
    actualState:
      output: 4
messages: []
hadErrors: false
```

## Parameters

### operands

The `sub()` function expects exactly two integers as input. The **operands** can be either an
integer or the output of any configuration function that returns an integer. Separate the
**operands** with a comma (`,`).

```yaml
Type:         integer
Required:     true
MinimumCount: 2
MaximumCount: 2
```

## Output

The `sub()` function returns an integer representing the difference of the **operands**.

```yaml
Type: integer
```

<!-- Link reference definitions -->
