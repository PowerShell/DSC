---
description: Reference for the 'false' DSC configuration document function
ms.date:     01/19/2025
ms.topic:    reference
title:       false
---

# false

## Synopsis

Returns the boolean value false.

## Syntax

```Syntax
false()
```

## Description

The `false()` function returns the boolean value `false`. This function takes no arguments and
always returns `false`. It's useful for providing explicit boolean values in configurations
or for logical operations.

## Examples

### Example 1 - Basic false value

This configuration demonstrates basic usage of the `false()` function.

```yaml
# false.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo false value
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[false()]"
```

```bash
dsc config get --file false.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo false value
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: false
messages: []
hadErrors: false
```

## Parameters

The `false()` function takes no arguments.

```yaml
Type:         none
Required:     false
MinimumCount: 0
MaximumCount: 0
```

## Output

The `false()` function always returns the boolean value `false`.

```yaml
Type: boolean
```

<!-- Link reference definitions -->
