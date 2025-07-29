---
description: Reference for the 'bool' DSC configuration document function
ms.date:     01/19/2025
ms.topic:    reference
title:       bool
---

# bool

## Synopsis

Converts a string or number to a boolean value.

## Syntax

```Syntax
bool(<value>)
```

## Description

The `bool()` function converts a string or number to a boolean value. For string arguments,
it accepts "true" (case-insensitive) which converts to `true`, and "false" (case-insensitive)
which converts to `false`. For numeric arguments, zero converts to `false` and any non-zero
value converts to `true`.

> [!NOTE]
> Any string argument other than `true` or `false` (case-insensitive) will raise a DSC error.

## Examples

### Example 1 - Convert string to boolean

This configuration demonstrates converting string values to boolean.

```yaml
# bool.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo bool from string
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        trueValue: "[bool('true')]"
        falseValue: "[bool('FALSE')]"
```

```bash
dsc config get --file bool.example.1.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0334711S
  name: Echo bool from string
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        trueValue: true
        falseValue: false
messages: []
hadErrors: false
```

### Example 2 - Convert number to boolean

This example shows the `bool()` function converting numeric values to boolean.

```yaml
# bool.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo bool from numbers
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        zeroIsFalse: "[bool(0)]"
        oneIsTrue: "[bool(1)]"
        negativeIsTrue: "[bool(-5)]"
        positiveIsTrue: "[bool(42)]"
```

```bash
dsc config get --file bool.example.2.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0323199S
  name: Echo bool from numbers
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        zeroIsFalse: false
        oneIsTrue: true
        negativeIsTrue: true
        positiveIsTrue: true
messages: []
hadErrors: false
```

## Parameters

### value

The `bool()` function requires a single argument that is either a string or number.

For strings, valid values are:
- "true" (case-insensitive) - converts to `true`
- "false" (case-insensitive) - converts to `false`

For numbers:
- 0 - converts to `false`
- Any non-zero value - converts to `true`

```yaml
Type:         [string, integer]
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `bool()` function returns a boolean value based on the input argument.

```yaml
Type: boolean
```
