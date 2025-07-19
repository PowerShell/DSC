---
description: Reference for the 'not' DSC configuration document function
ms.date:     01/19/2025
ms.topic:    reference
title:       not
---

# not

## Synopsis

Negates a boolean value.

## Syntax

```Syntax
not(<value>)
```

## Description

The `not()` function negates a boolean value, returning the logical opposite. If the input is
`true`, it returns `false`. If the input is `false`, it returns `true`. This function accepts
a single boolean argument.

## Examples

### Example 1 - Basic not operation

This configuration demonstrates basic usage of the `not()` function.

```yaml
# not.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo not operations
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        notTrue: "[not(true)]"
        notFalse: "[not(false)]"
```

```bash
dsc config get --file not.example.1.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0328813S
  name: Echo not operations
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        notTrue: false
        notFalse: true
```

## Parameters

### value

The `not()` function requires a single boolean argument.

```yaml
Type:         boolean
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `not()` function returns the logical opposite of the input boolean value.

```yaml
Type: boolean
```

<!-- Link reference definitions -->
