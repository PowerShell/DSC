---
description: Reference for the 'coalesce' DSC configuration document function
ms.date:     07/24/2025
ms.topic:    reference
title:       coalesce
---

# coalesce

## Synopsis

Returns the first non-null value from a list of arguments.

## Syntax

```Syntax
coalesce(<value1>, <value2>, ...)
```

## Description

The `coalesce()` function evaluates arguments from left to right and returns the first argument that is not null. This function is useful for providing fallback values when dealing with potentially null data.

If all arguments are null, the function returns null.

## Examples

### Example 1 - Basic coalesce with strings

The following example shows how to use the function with string values.

```yaml
# coalesce.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Coalesce strings
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: 
      firstNonNull:     "[coalesce(null, 'hello', 'world')]"
      allNull:          "[coalesce(null, null, null)]"
      firstNotNull:     "[coalesce('first', 'second', 'third')]"
```

```bash
dsc config get --file coalesce.example.1.dsc.config.yaml
```

```yaml
results:
- name: Coalesce strings
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        firstNonNull: hello
        allNull: null
        firstNotNull: first
messages: []
hadErrors: false
```

### Example 2 - Mixed data types

The following example shows how the function works with different data types.

```yaml
# coalesce.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Coalesce mixed types
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      numberFallback:   "[coalesce(null, 42)]"
      booleanFallback:  "[coalesce(null, null, true)]"
      stringToNumber:   "[coalesce(null, 123, 'fallback')]"
```

```bash
dsc config get --file coalesce.example.2.dsc.config.yaml
```

```yaml
results:
- name: Coalesce mixed types
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        numberFallback: 42
        booleanFallback: true
        stringToNumber: 123
messages: []
hadErrors: false
```

### Example 3 - Configuration fallbacks

The following example shows a practical use case for providing configuration defaults.

```yaml
# coalesce.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  customValue:
    type: string
    defaultValue: null
  timeout:
    type: int
    defaultValue: 0
resources:
- name: Configuration with fallbacks
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      configValue: "[coalesce(parameters('customValue'), 'default-config')]"
      timeout:     "[coalesce(parameters('timeout'), 30)]"
```

```bash
dsc config get --file coalesce.example.3.dsc.config.yaml
```

```yaml
results:
- name: Configuration with fallbacks
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        configValue: default-config
        timeout: 0
messages: []
hadErrors: false
```

## Parameters

### value1, value2, ...

The `coalesce()` function accepts one or more arguments of any type.
Arguments are evaluated from left to right, and the function returns the first non-null
value encountered.

```yaml
Type:         [any]
Required:     true
MinimumCount: 1
MaximumCount: unlimited
```

## Output

The `coalesce()` function returns the first non-null argument, or null if all arguments are null.
The return type matches the type of the first non-null argument.

```yaml
Type: [any]
```

<!-- Link reference definitions -->
