---
description: Reference for the 'intersection' DSC configuration document function
ms.date:     09/26/2025
ms.topic:    reference
title:       intersection
---

## Synopsis

Returns a single array or object with the common elements from the parameters.

## Syntax

```Syntax
intersection(value1, value2, ...)
```

## Description

The `intersection()` function takes two or more arrays or objects and returns
only the elements that exist in all of them. For arrays, it returns elements
that appear in every array. For objects, it returns key-value pairs where both
the key and value match across all objects.

All parameters must be the same type - either all arrays or all objects.
Results are deduplicated, meaning each element appears only once in the output.

Supported types:

- Arrays (elements compared by value)
- Objects (key-value pairs compared by deep equality)

## Examples

### Example 1 - Find common security groups across environments (arrays)

Use `intersection()` to identify security groups that are consistently applied
across development, staging, and production environments. This helps ensure
security policies are uniformly enforced. This example uses
[`createArray()`][01] to build the security group lists.

```yaml
# intersection.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Common Security Groups
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      commonGroups: "[intersection(createArray('admin-access', 'monitoring', 'backup'), createArray('monitoring', 'backup', 'web-access'), createArray('backup', 'monitoring', 'database'))]"
      twoEnvCommon: "[intersection(createArray('admin-access', 'monitoring'), createArray('monitoring', 'audit-log'))]"
```

```bash
dsc config get --file intersection.example.1.dsc.config.yaml
```

```yaml
results:
- name: Common Security Groups
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        commonGroups:
        - monitoring
        - backup
        twoEnvCommon:
        - monitoring
messages: []
hadErrors: false
```

### Example 2 - Identify shared configuration properties (objects)

Find configuration settings that are identical across multiple service
instances. This is useful for extracting common configuration into shared
templates or validating consistency. This example uses [`createObject()`][02]
to build configuration objects.

```yaml
# intersection.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Shared Config Properties
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      commonSettings: "[intersection(createObject('timeout', 30, 'retries', 3, 'region', 'us-east'), createObject('retries', 3, 'ssl', true, 'region', 'us-east'), createObject('region', 'us-east', 'retries', 3, 'logging', 'info'))]"
```

```bash
dsc config get --file intersection.example.2.dsc.config.yaml
```

```yaml
results:
- name: Shared Config Properties
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        commonSettings:
          region: us-east
          retries: 3
messages: []
hadErrors: false
```

### Example 3 - Find overlapping server capabilities (arrays with no matches)

Sometimes environments have no common elements, which is valuable information
for infrastructure planning. This example shows how `intersection()` handles
arrays with no shared elements.

```yaml
# intersection.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Server Capabilities
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      noOverlap: "[intersection(createArray('windows-iis', 'dotnet-core'), createArray('linux-apache', 'php', 'mysql'))]"
      someOverlap: "[intersection(createArray('docker', 'kubernetes', 'monitoring'), createArray('monitoring', 'logging', 'docker'))]"
```

```bash
dsc config get --file intersection.example.3.dsc.config.yaml
```

```yaml
results:
- name: Server Capabilities
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        noOverlap: []
        someOverlap:
        - docker
        - monitoring
messages: []
hadErrors: false
```

### Example 4 - Validate compliance across teams (objects)

Use `intersection()` to verify that critical compliance settings are identical
across different team configurations. Only settings with matching values will
appear in the result.

```yaml
# intersection.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Compliance Check
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      sharedCompliance: "[intersection(createObject('encryption', true, 'backup', 'daily', 'audit', true), createObject('audit', true, 'encryption', true, 'access', 'restricted'), createObject('encryption', true, 'audit', true, 'monitoring', 'enabled'))]"
```

```bash
dsc config get --file intersection.example.4.dsc.config.yaml
```

```yaml
results:
- name: Compliance Check
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        sharedCompliance:
          audit: true
          encryption: true
messages: []
hadErrors: false
```

## Parameters

### value1

The first array or object to compare. Required.

```yaml
Type:     array | object
Required: true
Position: 1
```

### value2

The second array or object to compare. Must be the same type as value1.
Required.

```yaml
Type:     array | object
Required: true
Position: 2
```

### Additional values

Additional arrays or objects to include in the intersection. All must be the
same type. Optional.

```yaml
Type:     array | object
Required: false
Position: 3+
```

## Output

Returns an array or object containing only the common elements. The return type
matches the input type.

```yaml
Type: array | object
```

## Related functions

- [`union()`][00] - Combines all elements from multiple arrays or objects
- [`contains()`][03] - Checks for presence in arrays/objects/strings
- [`createArray()`][01] - Creates an array from individual values
- [`createObject()`][02] - Creates an object from key-value pairs

<!-- Link reference definitions -->
[00]: ./union.md
[01]: ./createArray.md
[02]: ./createObject.md
[03]: ./contains.md
