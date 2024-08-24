---
description: Reference for the 'variables' DSC configuration document function
ms.date:     08/22/2024
ms.topic:    reference
title:       variables
---

# variables

## Synopsis

Returns the value of a configuration variable.

## Syntax

```Syntax
variables('<name>')
```

## Description

The `variables()` function returns the value of a specific variable. You must pass the name of
a valid variable. When using this function for a resource instance, DSC validates the instance
properties after this function runs and before calling the resource for the current operation. If
the referenced variable value is invalid for the property, DSC raises a validation error.

For more information about defining variables in a configuration document, see
[DSC Configuration document schema reference][01].

## Examples

### Example 1 - Use a variable as a resource instance property value

example synopsis

```yaml
# variables.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
variables:
  message: Hello, world!
resources:
  - name: Echo message variable
    type: Test/Echo
    properties:
      output: "[variables('message')]"
```

```bash
dsc config get --document variables.example.1.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0883345S
  name: Echo message variable
  type: Test/Echo
  result:
    actualState:
      output: Hello, world!
```

## Parameters

### name

The `variables()` function expects a single string as input, representing the name of the
variable to return. If no variable with the specified name is defined in the configuration
document, DSC raises an error during validation.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `variables()` function returns the value of the specified parameter.

```yaml
Type: [string, int, bool, object, array]
```

<!-- Link reference definitions -->

[01]: ../document.md#variables
