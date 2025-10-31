---
description: Reference for the 'tryIndexFromEnd' DSC configuration document function
ms.date:     01/29/2025
ms.topic:    reference
title:       tryIndexFromEnd
---

## Synopsis

Safely retrieves a value from an array by counting backward from the end without
throwing an error if the index is out of range.

## Syntax

```Syntax
tryIndexFromEnd(sourceArray, reverseIndex)
```

## Description

The `tryIndexFromEnd()` function provides a safe way to access array elements by
counting backward from the end using a one-based index. Unlike standard array
indexing that might fail with out-of-bounds errors, this function returns `null`
when the index is invalid or out of range.

This is particularly useful when working with dynamic arrays where the length
isn't known in advance, or when implementing fallback logic that needs to handle
missing data gracefully. The function uses a one-based index, meaning `1`
refers to the last element, `2` to the second-to-last, and so on.

The function returns `null` in the following cases:

- The reverse index is greater than the array length
- The reverse index is zero or negative
- The array is empty

## Examples

### Example 1 - Access recent deployment history safely

Use `tryIndexFromEnd()` to access recent deployment records when you're not
certain how many deployments have occurred. This is useful for rollback
scenarios where you want to retrieve the previous deployment without causing
errors if the history is empty or shorter than expected. This example uses
[`createArray()`][05] to build the deployment history and [`last()`][00] to get
the current deployment.

```yaml
# tryIndexFromEnd.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Deployment Rollback
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      currentDeployment: "[last(createArray('v1.0.0', 'v1.1.0', 'v1.2.0'))]"
      previousDeployment: "[tryIndexFromEnd(createArray('v1.0.0', 'v1.1.0', 'v1.2.0'), 2)]"
      fallbackDeployment: "[tryIndexFromEnd(createArray('v1.0.0', 'v1.1.0', 'v1.2.0'), 10)]"
```

```bash
dsc config get --file tryIndexFromEnd.example.1.dsc.config.yaml
```

```yaml
results:
- name: Deployment Rollback
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        currentDeployment: v1.2.0
        previousDeployment: v1.1.0
        fallbackDeployment: null
messages: []
hadErrors: false
```

The function returns `v1.1.0` for the second-to-last deployment, and `null` for
the non-existent 10th-from-last deployment, allowing your configuration to
handle missing data gracefully.

### Example 2 - Select backup retention with safe defaults

Use `tryIndexFromEnd()` to implement flexible backup retention policies that
adapt to available backups without failing when fewer backups exist than
expected. This example retrieves the third-most-recent backup if available. This
example uses [`parameters()`][06] to reference the backup timestamps array.

```yaml
# tryIndexFromEnd.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  backupTimestamps:
    type: array
    defaultValue:
      - 20250101
      - 20250108
      - 20250115
      - 20250122
      - 20250129
resources:
- name: Backup Retention
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      backups: "[parameters('backupTimestamps')]"
      retainAfter: "[tryIndexFromEnd(parameters('backupTimestamps'), 3)]"
      description: "Retain backups newer than the third-most-recent"
```

```bash
dsc config get --file tryIndexFromEnd.example.2.dsc.config.yaml
```

```yaml
results:
- name: Backup Retention
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        backups:
        - 20250101
        - 20250108
        - 20250115
        - 20250122
        - 20250129
        retainAfter: 20250115
        description: Retain backups newer than the third-most-recent
messages: []
hadErrors: false
```

The function safely returns `20250115` (the third-from-last backup), allowing
you to implement a retention policy that keeps the three most recent backups.

### Example 3 - Parse log levels from configuration arrays

Use `tryIndexFromEnd()` to access configuration values from arrays of varying
lengths. This is useful when configuration arrays might have different numbers
of elements across environments. This example uses [`parameters()`][06] to
reference the log level arrays.

```yaml
# tryIndexFromEnd.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  productionLevels:
    type: array
    defaultValue: [ERROR, WARN, INFO]
  devLevels:
    type: array
    defaultValue: [ERROR, WARN, INFO, DEBUG, TRACE]
resources:
- name: Log Configuration
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      productionLevels: "[parameters('productionLevels')]"
      devLevels: "[parameters('devLevels')]"
      prodThirdLevel: "[tryIndexFromEnd(parameters('productionLevels'), 3)]"
      devThirdLevel: "[tryIndexFromEnd(parameters('devLevels'), 3)]"
      prodFifthLevel: "[tryIndexFromEnd(parameters('productionLevels'), 5)]"
```

```bash
dsc config get --file tryIndexFromEnd.example.3.dsc.config.yaml
```

```yaml
results:
- name: Log Configuration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        productionLevels:
        - ERROR
        - WARN
        - INFO
        devLevels:
        - ERROR
        - WARN
        - INFO
        - DEBUG
        - TRACE
        prodThirdLevel: ERROR
        devThirdLevel: INFO
        prodFifthLevel: null
messages: []
hadErrors: false
```

The function safely handles arrays of different lengths, returning the
appropriate log level or `null` without throwing errors.

### Example 4 - Access region-specific configuration with fallback

Use `tryIndexFromEnd()` with [`coalesce()`][02] to implement fallback logic when
accessing configuration values from arrays that might have different lengths
across regions. This example shows how to safely access regional endpoints with
a default fallback.

```yaml
# tryIndexFromEnd.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Regional Endpoints
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      primaryRegion: "[createArray('us-east-1', 'us-west-2', 'eu-west-1')]"
      secondaryRegion: "[createArray('us-west-1')]"
      preferredPrimary: "[coalesce(tryIndexFromEnd(createArray('us-east-1', 'us-west-2', 'eu-west-1'), 2), 'us-east-1')]"
      preferredSecondary: "[coalesce(tryIndexFromEnd(createArray('us-west-1'), 2), 'us-west-1')]"
```

```bash
dsc config get --file tryIndexFromEnd.example.4.dsc.config.yaml
```

```yaml
results:
- name: Regional Endpoints
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        primaryRegion:
        - us-east-1
        - us-west-2
        - eu-west-1
        secondaryRegion:
        - us-west-1
        preferredPrimary: us-west-2
        preferredSecondary: us-west-1
messages: []
hadErrors: false
```

By combining `tryIndexFromEnd()` with `coalesce()`, you get robust fallback
behavior: `preferredPrimary` returns `us-west-2` (the second-to-last region),
while `preferredSecondary` falls back to the default `us-west-1` when the
second-to-last element doesn't exist.

## Parameters

### sourceArray

The array to retrieve the element from by counting backward from the end.
Required.

```yaml
Type:     array
Required: true
Position: 1
```

### reverseIndex

The one-based index from the end of the array. Must be a positive integer where
`1` refers to the last element, `2` to the second-to-last, and so on. Required.

```yaml
Type:     integer
Required: true
Position: 2
Minimum:  1
```

## Output

Returns the array element at the specified reverse index if the index is valid
(within array bounds). Returns `null` if the index is out of range, zero,
negative, or if the array is empty.

The return type matches the type of the element in the array.

```yaml
Type: any | null
```

## Errors

The function returns an error in the following cases:

- **Invalid source type**: The first argument is not an array
- **Invalid index type**: The second argument is not an integer

## Related functions

- [`last()`][00] - Returns the last element of an array (throws error if empty)
- [`first()`][01] - Returns the first element of an array or character of a string
- [`coalesce()`][02] - Returns the first non-null value from a list
- [`equals()`][03] - Compares two values for equality
- [`not()`][04] - Inverts a boolean value
- [`createArray()`][05] - Creates an array from provided values
- [`parameters()`][06] - Returns the value of a specified configuration parameter

<!-- Link reference definitions -->
[00]: ./last.md
[01]: ./first.md
[02]: ./coalesce.md
[03]: ./equals.md
[04]: ./not.md
[05]: ./createArray.md
[06]: ./parameters.md
