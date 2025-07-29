---
description: Reference for the 'true' DSC configuration document function
ms.date:     01/19/2025
ms.topic:    reference
title:       true
---

# true

## Synopsis

Returns the boolean value true.

## Syntax

```Syntax
true()
```

## Description

The `true()` function returns the boolean value `true`. This function takes no arguments and
always returns `true`. It's useful for providing explicit boolean values in configurations
or for logical operations.

## Examples

### Example 1 - Basic true value

This configuration demonstrates basic usage of the `true()` function.

```yaml
# true.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo true value
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[true()]"
```

```bash
dsc config get --file true.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo true value
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: true
messages: []
hadErrors: false
```

## Parameters

The `true()` function takes no arguments.

```yaml
Type:         none
Required:     false
MinimumCount: 0
MaximumCount: 0
```

## Output

The `true()` function always returns the boolean value `true`.

```yaml
Type: boolean
```

<!-- Link reference definitions -->
