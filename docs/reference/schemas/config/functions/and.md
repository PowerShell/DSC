---
description: Reference for the 'and' DSC configuration document function
ms.date:     01/19/2025
ms.topic:    reference
title:       and
---

# and

## Synopsis

Returns true if all arguments are true.

## Syntax

```Syntax
and(<arg1>, <arg2>, ...)
```

## Description

The `and()` function evaluates if all arguments are true. It takes two or more boolean arguments
and returns `true` only if every argument is `true`. If any argument is `false`, the function
returns `false`.

This function uses short-circuit evaluation, meaning it returns `false` as soon as it encounters
the first `false` argument without evaluating the remaining arguments.

## Examples

### Example 1 - Basic and operation

This configuration demonstrates basic usage of the `and()` function.

```yaml
# and.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo and result
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[and(true, true)]"
```

```bash
dsc config get --file and.example.1.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.1291763S
  name: Echo and result
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: true
messages: []
hadErrors: false
```

### Example 2 - And operation with false value

This example shows the `and()` function returning false when one argument is false.

```yaml
# and.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo and result with false
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[and(true, false, true)]"
```

```bash
dsc config get --file and.example.2.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0329292S
  name: Echo and result with false
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: false
messages: []
hadErrors: false
```

### Example 3 - And operation with multiple conditions

This configuration uses the `and()` function with multiple boolean expressions.

```yaml
# and.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo complex and operation
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[and(equals(5, 5), equals('hello', 'hello'), true)]"
```

```bash
dsc config get --file and.example.3.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0514415S
  name: Echo complex and operation
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: true
messages: []
hadErrors: false
```

## Parameters

### arguments

The `and()` function requires two or more boolean arguments.

```yaml
Type:         boolean
Required:     true
MinimumCount: 2
MaximumCount: 18446744073709551615
```

## Output

The `and()` function returns `true` if all arguments are `true`, otherwise it returns `false`.

```yaml
Type: boolean
```

<!-- Link reference definitions -->
