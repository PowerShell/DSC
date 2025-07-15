---
description: Reference for the 'if' DSC configuration document function
ms.date:     07/02/2025
ms.topic:    reference
title:       if
---

# if

## Synopsis

Returns a value based on whether a condition is true or false.

## Syntax

```Syntax
if(<condition>, <trueValue>, <falseValue>)
```

## Description

The `if()` function returns a value based on whether a condition is true or false. You can use this
function to conditionally use different values in a configuration document.

## Examples

### Example 1 - Returning values

This example shows the returning of values when the conditional evaluates to `true` and `false`.

```yaml
# if.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Show return for true and false
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      whenTrue:  "[if(equals('a', 'a'), 1, 2)]"
      whenFalse: "[if(equals('a', 'b'), 1, 2)]"
```

```bash
dsc config get --file if.example.1.dsc.config.yaml
```

```yaml
results:
- name: Show return for true and false
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        whenTrue:  1
        whenFalse: 2
messages: []
hadErrors: false
```

## Parameters

### condition

The `if()` function expects the first parameter to be a boolean value or an expression that
evaluates to a boolean value. When this parameter is `true`, the `if()` function returns the
`trueValue`. When this parameter is `false`, the function returns the `falseValue`.

```yaml
Type:         boolean
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### trueValue

The `if()` function expects the second parameter to be the value to return when the `condition`
parameter evaluates to `true`. This parameter may be a literal value or an expression that
evaluates to a string, integer, boolean, object, or array value.

```yaml
Type:         [string, int, bool, object, array]
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### falseValue

The `if()` function expects the third parameter to be the value to return when the `condition`
parameter evaluates to `false`. This parameter may be a literal value or an expression that
evaluates to a string, integer, boolean, object, or array value.

```yaml
Type:         [string, int, bool, object, array]
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The function returns either the `trueValue` or `falseValue` depending on whether the `condition`
parameter evaluates to `true` or `false`.

```yaml
Type: [string, int, bool, object, array]
```

<!-- Link reference definitions -->
