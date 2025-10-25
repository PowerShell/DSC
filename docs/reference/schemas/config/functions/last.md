---
description: Reference for the 'last' DSC configuration document function
ms.date:     01/25/2025
ms.topic:    reference
title:       last
---

## Synopsis

Returns the last element of an array, or the last character of a string.

## Syntax

```Syntax
last(arg)
```

## Description

The `last()` function returns the final element from an array or the final
character from a string. This is useful when you need to access the most recent
item in a sequence, the final stage in a deployment pipeline, or the last
character in a configuration value.

For arrays, it returns the element at index `length - 1`. For strings, it
returns the last character as a string. If the input is empty, an error is
returned.

## Examples

### Example 1 - Extract the final deployment stage (array of strings)

Use `last()` to retrieve the final stage in a multi-stage deployment pipeline.
This helps you identify which environment or phase should receive special
handling, such as extended health checks or manual approval gates. This example
uses [`createArray()`][01] to build the deployment stages.

```yaml
# last.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Deployment Pipeline
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      finalStage: "[last(createArray('dev', 'test', 'staging', 'production'))]"
      requiresApproval: true
```

```bash
dsc config get --file last.example.1.dsc.config.yaml
```

```yaml
results:
- name: Deployment Pipeline
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        finalStage: production
        requiresApproval: true
messages: []
hadErrors: false
```

This identifies `production` as the final stage, allowing you to apply
production-specific policies or validations.

### Example 2 - Get the last character of a configuration string

Use `last()` to extract the final character from a string value. This is useful
for parsing identifiers, checking suffixes, or validating format conventions
like version numbers or region codes.

```yaml
# last.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Region Identifier
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      regionCode: us-west-2
      zoneSuffix: "[last('us-west-2')]"
      description: "Zone suffix extracted from region code"
```

```bash
dsc config get --file last.example.2.dsc.config.yaml
```

```yaml
results:
- name: Region Identifier
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        regionCode: us-west-2
        zoneSuffix: '2'
        description: Zone suffix extracted from region code
messages: []
hadErrors: false
```

The function returns `'2'` as a single-character string, representing the zone
suffix in the region identifier.

### Example 3 - Identify the most recent backup (array of numbers)

Use `last()` with numerical arrays to find the most recent timestamp or version
number. This example shows how to select the latest backup from a sorted list
of timestamps. This example uses [`createArray()`][01] to build the backup
timestamps.

```yaml
# last.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Backup Selection
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      availableBackups: "[createArray(1704067200, 1704153600, 1704240000, 1704326400)]"
      latestBackup: "[last(createArray(1704067200, 1704153600, 1704240000, 1704326400))]"
      description: "Most recent backup timestamp (Unix epoch)"
```

```bash
dsc config get --file last.example.3.dsc.config.yaml
```

```yaml
results:
- name: Backup Selection
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        availableBackups:
        - 1704067200
        - 1704153600
        - 1704240000
        - 1704326400
        latestBackup: 1704326400
        description: Most recent backup timestamp (Unix epoch)
messages: []
hadErrors: false
```

The function returns `1704326400`, which represents the most recent backup in
the chronologically sorted array.

### Example 4 - Combine with other functions for complex logic

Use `last()` together with [`split()`][02] to extract file extensions or path
components. This example demonstrates parsing a filename to get its extension.

```yaml
# last.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: File Extension Parser
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      filename: config.production.yaml
      extension: "[last(split('config.production.yaml', '.'))]"
```

```bash
dsc config get --file last.example.4.dsc.config.yaml
```

```yaml
results:
- name: File Extension Parser
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        filename: config.production.yaml
        extension: yaml
messages: []
hadErrors: false
```

By combining `split()` and `last()`, you can extract the `yaml` extension from
the full filename.

## Parameters

### arg

The array or string to get the last element or character from. Required.

```yaml
Type:     array | string
Required: true
Position: 1
```

## Output

Returns the last element of the array (preserving its original type) or the
last character as a string. For arrays, the return type matches the element
type. For strings, returns a single-character string.

```yaml
Type: any | string
```

## Errors

The function returns an error in the following cases:

- **Empty array**: The input array has no elements
- **Empty string**: The input string has no characters
- **Invalid type**: The argument is not an array or string

## Related functions

- [`first()`][00] - Returns the first element of an array or character of a string
- [`split()`][02] - Splits a string into an array
- [`createArray()`][01] - Creates an array from provided values

<!-- Link reference definitions -->
[00]: ./first.md
[01]: ./createArray.md
[02]: ./split.md
