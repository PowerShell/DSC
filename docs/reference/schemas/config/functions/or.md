---
description: Reference for the 'or' DSC configuration document function
ms.date:     01/19/2025
ms.topic:    reference
title:       or
---

# or

## Synopsis

Returns true if any arguments are true.

## Syntax

```Syntax
or(<arg1>, <arg2>, ...)
```

## Description

The `or()` function evaluates if any arguments are true. It takes two or more boolean arguments
and returns `true` if at least one argument is `true`. If all arguments are `false`, the function
returns `false`.

This function uses short-circuit evaluation, meaning it returns `true` as soon as it encounters
the first `true` argument without evaluating the remaining arguments.

## Examples

### Example 1 - Basic or operation

This configuration demonstrates basic usage of the `or()` function.

```yaml
# or.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo or result
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[or(false, true)]"
```

```bash
dsc config get --file or.example.1.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0329859S
  name: Echo or result
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: true
messages: []
hadErrors: false
```

### Example 2 - Or operation with all false values

This example shows the `or()` function returning false when all arguments are false.

```yaml
# or.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo or result all false
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[or(false, false, false)]"
```

```bash
dsc config get --file or.example.2.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0320911S
  name: Echo or result all false
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: false
messages: []
hadErrors: false
```

### Example 3 - Or operation with multiple conditions

This configuration uses the `or()` function with multiple boolean expressions.

```yaml
# or.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo complex or operation
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[or(equals(5, 10), equals('hello', 'world'), true)]"
```

```bash
dsc config get --file or.example.3.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0324607S
  name: Echo complex or operation
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: true
messages: []
hadErrors: false
```

## Parameters

### arguments

The `or()` function requires two or more boolean arguments.

```yaml
Type:         boolean
Required:     true
MinimumCount: 2
MaximumCount: 18446744073709551615
```

## Output

The `or()` function returns `true` if any argument is `true`, otherwise it returns `false`.

```yaml
Type: boolean
```

<!-- Link reference definitions -->
