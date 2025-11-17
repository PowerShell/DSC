---
description: Reference for the 'startsWith' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       startsWith
---

# startsWith

## Synopsis

Determines whether a string starts with the specified prefix.

## Syntax

```Syntax
startsWith(<string>, <prefix>)
```

## Description

The `startsWith()` function returns `true` if the first string starts with the
specified prefix. Comparison is case-sensitive. Use it for conditional logic in
configuration documents such as grouping resource names, validating identifiers,
or routing operations based on naming conventions.

## Examples

### Example 1 - Validate resource naming convention

The following example checks if a resource name follows a standard prefix.

```yaml
# startswith.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  resourceName:
    type: string
    defaultValue: svc-api-west
resources:
- name: Resource naming check
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      hasSvcPrefix: "[startsWith(parameters('resourceName'), 'svc-')]"
```

```bash
dsc config get --file startswith.example.1.dsc.config.yaml
```

```yaml
results:
- name: Resource naming check
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hasSvcPrefix: true
messages: []
hadErrors: false
```

### Example 2 - Conditional routing

The following example shows using `startsWith()` with `if()` to categorize a
service by its name prefix.

```yaml
# startswith.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  serviceName:
    type: string
    defaultValue: api-orders
resources:
- name: Service classification
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      classification: "[if(startsWith(parameters('serviceName'), 'api-'), 'API Service', 'Other Service')]"
```

```bash
dsc config get --file startswith.example.2.dsc.config.yaml
```

```yaml
results:
- name: Service classification
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        classification: API Service
messages: []
hadErrors: false
```

### Example 3 - Multi-prefix evaluation

The following example evaluates multiple possible prefixes.

```yaml
# startswith.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  name:
    type: string
    defaultValue: db-primary-01
resources:
- name: Prefix grouping
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      isDb: "[startsWith(parameters('name'), 'db-')]"
      isApi: "[startsWith(parameters('name'), 'api-')]"
      isCache: "[startsWith(parameters('name'), 'cache-')]"
```

```bash
dsc config get --file startswith.example.3.dsc.config.yaml
```

```yaml
results:
- name: Prefix grouping
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        isDb: true
        isApi: false
        isCache: false
messages: []
hadErrors: false
```

## Parameters

### string

The input string to evaluate.

```yaml
Type:     string
Required: true
Position: 1
```

### prefix

The prefix string to test for.

```yaml
Type:     string
Required: true
Position: 2
```

## Output

The `startsWith()` function returns a boolean value indicating whether the
input string starts with the specified prefix.

```yaml
Type: bool
```

## Related functions

- [`endsWith()`][00] - Determines whether a string ends with a suffix
- [`concat()`][01] - Concatenates strings together
- [`if()`][02] - Returns values based on a condition
- [`string()`][03] - Converts values to strings

<!-- Link reference definitions -->
[00]: ./endsWith.md
[01]: ./concat.md
[02]: ./if.md
[03]: ./string.md
