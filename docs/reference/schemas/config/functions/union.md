---
description: Reference for the 'union' DSC configuration document function
ms.date:     08/08/2025
ms.topic:    reference
title:       union
---

# union

## Synopsis

Combines two or more arrays or objects, returning a single collection with
unique elements or merged properties.

## Syntax

```Syntax
union(<collection1>, <collection2>, [collection3], ...)
```

## Description

The `union()` function combines multiple collections into a single collection.
For arrays, it returns a new array containing all unique elements from the
input arrays, preserving order and removing duplicates. For objects, it
merges properties from all input objects, with later objects overriding
properties from earlier ones when keys conflict.

All input parameters must be of the same type (all arrays or all objects).
Mixing arrays and objects will result in an error.

## Examples

### Example 1 - Union of arrays

The following example shows how to combine multiple arrays into one.

```yaml
# union.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  serverGroup1:
    type: array
    defaultValue:
    - web01
    - web02
    - db01
  serverGroup2:
    type: array
    defaultValue:
    - web02
    - web03
    - cache01
  serverGroup3:
    type: array
    defaultValue:
    - db01
    - backup01
resources:
- name: Combine server groups
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      allServers: "[union(parameters('serverGroup1'), parameters('serverGroup2'))]"
      threeGroups: "[union(parameters('serverGroup1'), parameters('serverGroup2'), parameters('serverGroup3'))]"
```

```bash
dsc config get --file union.example.1.dsc.config.yaml
```

```yaml
results:
- name: Combine server groups
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        allServers:
        - web01
        - web02
        - db01
        - web03
        - cache01
        threeGroups:
        - web01
        - web02
        - db01
        - web03
        - cache01
        - backup01
messages: []
hadErrors: false
```

### Example 2 - Union of objects

The following example shows how to merge multiple objects.

```yaml
# union.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  defaultConfig:
    type: object
    defaultValue:
      timeout: 30
      retries: 3
      debug: false
  userConfig:
    type: object
    defaultValue:
      timeout: 60
      logLevel: info
  envConfig:
    type: object
    defaultValue:
      debug: true
      environment: production
resources:
- name: Merge configurations
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      finalConfig: "[union(parameters('defaultConfig'), parameters('userConfig'), parameters('envConfig'))]"
```

```bash
dsc config get --file union.example.2.dsc.config.yaml
```

```yaml
results:
- name: Merge configurations
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        finalConfig:
          timeout: 60
          retries: 3
          debug: true
          logLevel: info
          environment: production
messages: []
hadErrors: false
```

### Example 3 - Union with duplicate arrays

The following example shows how union handles duplicate values in arrays.

```yaml
# union.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  permissions1:
    type: array
    defaultValue:
    - read
    - write
    - execute
  permissions2:
    type: array
    defaultValue:
    - read
    - admin
    - delete
resources:
- name: Combine permissions
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      uniquePermissions: "[union(parameters('permissions1'), parameters('permissions2'))]"
      selfUnion: "[union(parameters('permissions1'), parameters('permissions1'))]"
```

```bash
dsc config get --file union.example.3.dsc.config.yaml
```

```yaml
results:
- name: Combine permissions
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        uniquePermissions:
        - read
        - write
        - execute
        - admin
        - delete
        selfUnion:
        - read
        - write
        - execute
messages: []
hadErrors: false
```

## Parameters

### collection1

The first collection to union.

```yaml
Type:         [array, object]
Required:     true
```

### collection2

The second collection to union. Must be the same type as collection1.

```yaml
Type:         [array, object]
Required:     true
```

### Additional collections

Additional collections to union (optional). All must be the same type as
collection1.

```yaml
Type:         [array, object]
Required:     false
```

The `union()` function requires at least two input values and accepts
additional values. All input values must be of the same type (all arrays or
all objects).

For arrays, the function preserves the order of elements and removes
duplicates. For objects, properties from later objects override properties
from earlier objects when there are key conflicts.

## Output

The `union()` function returns a single collection of the same type as the
input collections.

```yaml
Type: [array, object]
```

<!-- Link reference definitions -->
