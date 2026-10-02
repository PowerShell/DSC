---
description: Reference for the 'if' DSC configuration document function
ms.date:     09/01/2026
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

The values to return must be strings, integers, objects, or arrays. The function doesn't accept
boolean or null values for `trueValue` or `falseValue`. For more information, see
[Error conditions](#error-conditions).

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
evaluates to a string, integer, object, or array value. Boolean and null values aren't
accepted for this parameter.

```yaml
Type:         [string, int, object, array]
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### falseValue

The `if()` function expects the third parameter to be the value to return when the `condition`
parameter evaluates to `false`. This parameter may be a literal value or an expression that
evaluates to a string, integer, object, or array value. Boolean and null values aren't
accepted for this parameter.

```yaml
Type:         [string, int, object, array]
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The function returns either the `trueValue` or `falseValue` depending on whether the `condition`
parameter evaluates to `true` or `false`.

```yaml
Type: [string, int, object, array]
```

## Error conditions

DSC raises an error when the `trueValue` or `falseValue` parameter is a boolean or null value.
For example, the expression `[if(true(), true(), false())]` fails with the error
`Function 'if' does not accept boolean arguments, accepted types are: String, Number, Array,
Object`.

To return a boolean value based on a condition, use the condition expression directly instead of
wrapping it in `if()`.

<!-- Link reference definitions -->
