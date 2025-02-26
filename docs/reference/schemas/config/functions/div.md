---
description: Reference for the 'div' DSC configuration document function
ms.date:     02/28/2025
ms.topic:    reference
title:       div
---

# div

## Synopsis

Returns the quotient for the division of two integers.

## Syntax

```Syntax
div(<operands>)
```

## Description

The `div()` function returns the quotient for the division of two integers. If the result of the
division isn't an integer, the function returns the value of the division rounded down to the
nearest integer.

## Examples

### Example 1 - Divide two integers

This example document shows how you can use the `div()` function to return the division of two
integers.

```yaml
# div.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Dividing integers
  type: Test/Echo
  properties:
    output: "[div(6,3)]"
```

```bash
dsc config get --document div.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Dividing integers
  type: Test/Echo
  result:
    actualState:
      output: 2
messages: []
hadErrors: false
```

### Example 2 - Divide output of nested functions

This example document shows how you can use the `div()` function to divide the outputs of nested
configuration functions. Because the outputs are 14 and 5, the final result is 2. DSC returns the
full integer value without the remainder. It doesn't round the result up to 3.

```yaml
# div.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Dividing nested functions
  type: Test/Echo
  properties:
    output: "[div(mul(7,2), add(4,1))]"
```

```bash
dsc config get --document div.example.2.dsc.config.yaml
```

```yaml
results:
- name: Dividing nested functions
  type: Test/Echo
  result:
    actualState:
      output: 2
messages: []
hadErrors: false
```

## Parameters

### operands

The `div()` function expects exactly two integers as input. The **operands** can be either an integer
or the output of any configuration function that returns an integer. The function divides the
first operand by the second operand. Separate the **operands** with a comma (`,`).

```yaml
Type:         integer
Required:     true
MinimumCount: 2
MaximumCount: 2
```

## Output

The `div()` function returns an integer value representing the division of the first operand by the
second operand. If the division result isn't an integer, the function returns the integer value of
the result without the fractional remainder.

```yaml
Type: integer
```

<!-- Link reference definitions -->
