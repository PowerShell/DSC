---
description: Reference for the 'envvar' DSC configuration document function
ms.date:     03/01/2024
ms.topic:    reference
title:       envvar
---

# envvar

## Synopsis

Returns the value of an environment variable.

## Syntax

```Syntax
envvar(<variableName>)
```

## Description

The `envvar()` function returns the value of an environment variable as a string. If the
environment variable doesn't exist, DSC raises an error.

## Examples

### Example 1 - Reference DSCConfigRoot in a configuration

When you use the `--path` option to specify a configuration document for any of the `dsc config *`
commands, DSC automatically creates the `DSCConfigRoot` environment variable and sets the value to
the parent folder of the specified configuration document. For more information, see
[dsc config command reference][01].

This configuration echoes that folder with the `Test/Echo` resource.

```yaml
# ./examples/envvar.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
  - name: Echo 'DSCConfigRoot' in envvar
    type: Test/Echo
    properties:
      output: "[envvar('DSCConfigRoot')]"
```

```bash
dsc config get --path ~/dsc/examples/envvar.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo DSCConfigRoot
  type: Test/Echo
  result:
    actualState:
      output: ~/dsc/examples
messages: []
hadErrors: false
```

## Parameters

### variableName

The `envvar()` function expects a single string representing the name of the environment variable
to use. If the value isn't a string, DSC raises an error when validating the configuration
document. If the environment variable named by the input doesn't exist, DSC raises an error.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `envvar()` function returns the value of the environment variable specified with the
**variableName** parameter.

```yaml
Type: string
```

[01]: ../../../cli/config/command.md#environment-variables
