---
description: Reference for the 'endsWith' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       endsWith
---

# endsWith

## Synopsis

Determines whether a string ends with the specified suffix.

## Syntax

```Syntax
endsWith(<string>, <suffix>)
```

## Description

The `endsWith()` function returns `true` if the first string ends with the
specified suffix. Comparison is case-sensitive. Use it for conditional logic in
configuration documents such as matching file extensions, environment name
suffixes, or resource identifiers.

## Examples

### Example 1 - Check a file extension

The following example checks if a specified filename ends with `.log`.

```yaml
# endswith.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  fileName:
    type: string
    defaultValue: application.log
resources:
- name: Check file extension
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      isLog: "[endsWith(parameters('fileName'), '.log')]"
```

```bash
dsc config get --file endswith.example.1.dsc.config.yaml
```

```yaml
results:
- name: Check file extension
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        isLog: true
messages: []
hadErrors: false
```

### Example 2 - Conditional environment handling

The following example uses `endsWith()` to build a message when an environment
name ends with `-prod`.

```yaml
# endswith.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  environment:
    type: string
    defaultValue: web-prod
resources:
- name: Environment classification
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      classification: "[if(endsWith(parameters('environment'), '-prod'), 'Production', 'Non-production')]"
```

```bash
dsc config get --file endswith.example.2.dsc.config.yaml
```

```yaml
results:
- name: Environment classification
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        classification: Production
messages: []
hadErrors: false
```

### Example 3 - Filter resource identifiers

The following example shows checking multiple suffixes by combining conditions.

```yaml
# endswith.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  resourceId:
    type: string
    defaultValue: storage-westus-01
resources:
- name: Identify resource segment
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      isRegional: "[endsWith(parameters('resourceId'), '-01')]"
      endsWithWest: "[endsWith(parameters('resourceId'), 'westus-01')]"
      endsWithEast: "[endsWith(parameters('resourceId'), 'eastus-01')]"
```

```bash
dsc config get --file endswith.example.3.dsc.config.yaml
```

```yaml
results:
- name: Identify resource segment
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        isRegional: true
        endsWithWest: true
        endsWithEast: false
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

### suffix

The suffix string to test for.

```yaml
Type:     string
Required: true
Position: 2
```

## Output

The `endsWith()` function returns a boolean value indicating whether the input
string ends with the specified suffix.

```yaml
Type: bool
```

## Related functions

- [`startsWith()`][00] - Determines whether a string starts with a prefix
- [`concat()`][01] - Concatenates strings together
- [`if()`][02] - Returns values based on a condition
- [`string()`][03] - Converts values to strings

<!-- Link reference definitions -->
[00]: ./startsWith.md
[01]: ./concat.md
[02]: ./if.md
[03]: ./string.md
